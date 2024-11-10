// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{CloudHosts, HostParameters, HostResourceId};
use crate::common::{CubConfig, Error};
use crate::datacenter::CloudDatacenter;
use crate::log::StringLogger;
use crate::time_id::ID64;
use async_trait::async_trait;
use hyper::{http::HeaderValue, HeaderMap, StatusCode};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use std::num::NonZeroU64;
use std::time::Duration;

/// Linode virtual host.
pub struct LinodeHosts {
    authorized_ssh_key: String,
    client: Client,
    debug: bool,
    firewall_ids: HashMap<String, usize>,
    swap_size: Option<usize>,
}

impl LinodeHosts {
    const TIMEOUT_SECS: u64 = 5;

    fn compute_hash<T: Hash + ?Sized>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    /// Create a `CloudHosts` for Linode.
    pub fn new(cub_config: &CubConfig) -> Self {
        #[derive(Deserialize)]
        struct LinodeConfig {
            authorized_ssh_key: String,
            firewall_ids: HashMap<String, HostResourceId>,
            personal_access_token: String,
            swap_size: Option<usize>,
        }
        #[derive(Deserialize)]
        struct ConfigToml {
            linode: LinodeConfig,
        }
        let ConfigToml {
            linode:
                LinodeConfig {
                    authorized_ssh_key,
                    firewall_ids,
                    personal_access_token,
                    swap_size,
                },
        } = cub_config.get().expect("linode.toml");

        let firewall_ids: HashMap<_, _> = firewall_ids
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    Self::strip_resource_id_prefix(&v)
                        .unwrap()
                        .parse::<usize>()
                        .expect("firewall ID must be an unsigned integer"),
                )
            })
            .collect();

        let mut default_headers = HeaderMap::new();

        default_headers.insert(
            reqwest::header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", personal_access_token)).unwrap(),
        );
        default_headers.insert(
            reqwest::header::USER_AGENT,
            HeaderValue::from_str("softbear cloud control").unwrap(),
        );

        Self {
            authorized_ssh_key,
            client: Client::builder()
                .timeout(Duration::from_secs(Self::TIMEOUT_SECS))
                .default_headers(default_headers)
                .http1_only()
                .build()
                .unwrap(),
            debug: true,
            firewall_ids,
            swap_size,
        }
    }

    /// Create a `LinodeScript` for Linode.
    /// It is OK for more than one script to have the same label.
    pub async fn create_script(&self, label: &str, script: &str) -> Result<HostResourceId, Error> {
        let hash = Self::compute_hash(script);
        let logger = StringLogger::new(self.debug);
        logger.trace(format!("create linode script {} {}", label, hash));
        let endpoint = format!("https://api.linode.com/v4/linode/stackscripts");
        let record = LinodeScript {
            label: label.to_string(),
            description: None,
            images: vec![LINODE_IMAGE.to_string()],
            is_public: false,
            rev_note: None,
            script: script.to_string(),
        };
        let request = self.client.post(&endpoint);
        let request = request.json(&record).build().map_err(Self::map_error)?;
        logger.trace(format!("{record:?}")); // For now.
        let response = self
            .client
            .execute(request)
            .await
            .map_err(Self::map_error)?;
        let result = response.text().await.map_err(Self::map_error)?;
        let LinodeScriptResponse { id, .. } = Self::parse_result(&result)?;
        Ok(HostResourceId(format!("{LINODE_PROVIDER_NAME}/{id}")))
    }

    /// Delete a `LinodeScript` for Linode.
    pub async fn delete_script(&self, resource_id: &HostResourceId) -> Result<(), Error> {
        let script_id = Self::strip_resource_id_prefix(resource_id)?;
        let endpoint = format!("https://api.linode.com/v4/linode/stackscripts/{script_id}");
        let request = self
            .client
            .delete(&endpoint)
            .build()
            .map_err(Self::map_error)?;
        self.client
            .execute(request)
            .await
            .map_err(Self::map_error)?;
        Ok(())
    }

    /// List all `LinodeScript` for Linode.
    pub async fn list_scripts(&self) -> Result<Vec<(HostResourceId, String)>, Error> {
        let endpoint = format!("https://api.linode.com/v4/linode/stackscripts");
        let request = self.client.get(&endpoint);
        let response = request.send().await.map_err(Self::map_error)?;
        let result = response.text().await.map_err(Self::map_error)?;
        let list: ListLinodeScriptsResponse = Self::parse_result(&result)?;
        Ok(list
            .data
            .into_iter()
            .map(
                |LinodeScriptResponse {
                     id,
                     record: LinodeScript { label, .. },
                     ..
                 }| {
                    (
                        HostResourceId(format!("{LINODE_PROVIDER_NAME}/{id}")),
                        label,
                    )
                },
            )
            .collect::<Vec<_>>())
    }

    fn map_error(e: reqwest::Error) -> Error {
        Error::Http(StatusCode::FAILED_DEPENDENCY, format!("{}", e))
    }

    fn parse_result<'a, T: Deserialize<'a>>(text: &'a String) -> Result<T, Error> {
        match serde_json::from_str(&text) {
            Ok(response) => Ok(response),
            Err(_) => {
                #[derive(Deserialize)]
                struct LinodeReason {
                    field: String,
                    reason: String,
                }
                #[derive(Deserialize)]
                struct LinodeError {
                    errors: Vec<LinodeReason>,
                }
                match serde_json::from_str(&text) {
                    Ok(LinodeError { errors }) => {
                        let r = errors
                            .into_iter()
                            .next()
                            .map(|LinodeReason { field, reason }| format!("{field}: {reason}"));
                        Err(Error::Http(
                            StatusCode::FAILED_DEPENDENCY,
                            format!("linode error: {r:?}"),
                        ))
                    }
                    Err(_) => Err(Error::Http(
                        StatusCode::FAILED_DEPENDENCY,
                        format!("cannot parse linode error: {text}"),
                    )),
                }
            }
        }
    }

    fn strip_resource_id_prefix(resource_id: &HostResourceId) -> Result<String, Error> {
        let mut split = resource_id.0.splitn(2, '/');
        if split
            .next()
            .map(|s| s != LINODE_PROVIDER_NAME)
            .unwrap_or(true)
        {
            Err(Error::Http(
                StatusCode::NOT_ACCEPTABLE,
                format!("{}: expected 'linode' prefix in resource ID", resource_id.0),
            ))
        } else if let Some(payload) = split.next() {
            Ok(payload.to_string())
        } else {
            Err(Error::Http(
                StatusCode::NOT_ACCEPTABLE,
                format!("{}: invalid cloud resource ID", resource_id.0),
            ))
        }
    }
}

#[async_trait]
impl CloudHosts for LinodeHosts {
    async fn create_host(
        &self,
        label: &str,
        group: Option<&str>,
        hostname: &str,
        datacenter: CloudDatacenter,
        script: &str,
        parameters: Option<HostParameters>,
    ) -> Result<(HostResourceId, IpAddr), Error> {
        let default_firewall_name = "default".to_string();
        let firewall_parameter_name = "firewall_name".to_string();
        let firewall_id = if let Some(firewall_name) = parameters
            .as_ref()
            .and_then(|HostParameters(p)| p.get(&firewall_parameter_name))
        {
            Some(self.firewall_ids.get(firewall_name).ok_or(Error::Http(
                StatusCode::NOT_FOUND,
                format!("{firewall_name}: firewall not found"),
            ))?)
        } else {
            self.firewall_ids.get(&default_firewall_name)
        }
        .copied();
        let script = script.replace("{{hostname}}", hostname);
        let hash = Self::compute_hash(&script);
        let logger = StringLogger::new(self.debug);

        logger.trace(format!(
            "create linode instance {label} ({hostname}) in {datacenter:?} with script {hash}"
        ));

        let resource_id = self
            .create_script(&format!("Linode {label} Script {hash}"), &script)
            .await?;
        let script_id: usize =
            if let Ok(script_id) = Self::strip_resource_id_prefix(&resource_id)?.parse() {
                script_id
            } else {
                return Err(Error::Http(
                    StatusCode::NOT_ACCEPTABLE,
                    format!("{resource_id}: not a valid script ID"),
                ));
            };

        let r: NonZeroU64 = ID64::<0>::generate().into();
        let root_pass = Some(format!("aA!@{r}$%zZ"));
        let endpoint = format!("https://api.linode.com/v4/linode/instances");

        let record = LinodeInstance {
            authorized_keys: Some(vec![self.authorized_ssh_key.clone()]),
            image: LINODE_IMAGE.to_string(),
            label: label.to_string(),
            tags: group.into_iter().map(|s| s.to_owned()).collect(),
            region: datacenter.to_linode_region()?,
            root_pass,
            stackscript_id: Some(script_id),
            firewall_id,
            private_ip: false,
            linode_type: LINODE_TYPE.to_string(),
            swap_size: if let Some(swap_size) = self.swap_size {
                Some(swap_size)
            } else {
                Some(SWAP_SIZE_MB)
            },
        };
        let request = self.client.post(&endpoint);
        let request = request.json(&record).build().map_err(Self::map_error)?;
        logger.trace(format!("{record:?}")); // For now.
        let response = self
            .client
            .execute(request)
            .await
            .map_err(Self::map_error)?;
        let result = response.text().await.map_err(Self::map_error)?;
        let LinodeInstanceResponse {
            id: host_id, ipv4, ..
        } = Self::parse_result(&result)?;
        let ip_addr: IpAddr = ipv4
            .first()
            .and_then(|s| s.parse().ok())
            .ok_or(Error::Http(
                StatusCode::FAILED_DEPENDENCY,
                format!("{ipv4:?} does not contain an IP address"),
            ))?;
        Ok((
            HostResourceId(format!("{LINODE_PROVIDER_NAME}/{host_id}/{script_id}")),
            ip_addr,
        ))
    }

    async fn delete_host(&self, resource_id: &HostResourceId) -> Result<(), Error> {
        let sans_prefix = Self::strip_resource_id_prefix(resource_id)?;
        let mut split = sans_prefix.splitn(2, '/');
        let host_id: usize = split.next().and_then(|s| s.parse().ok()).ok_or_else(|| {
            Error::Http(
                StatusCode::NOT_ACCEPTABLE,
                format!("{resource_id}: does not contain a host_id"),
            )
        })?;

        let script_id: Option<usize> = split.next().and_then(|s| s.parse::<usize>().ok());
        if let Some(script_id) = script_id {
            self.delete_script(&HostResourceId(format!(
                "{LINODE_PROVIDER_NAME}/{script_id}"
            )))
            .await?;
        }

        let endpoint = format!("https://api.linode.com/v4/linode/instances/{host_id}");
        let request = self
            .client
            .delete(&endpoint)
            .build()
            .map_err(Self::map_error)?;
        self.client
            .execute(request)
            .await
            .map_err(Self::map_error)?;
        Ok(())
    }

    /// List datacenters.
    async fn list_datacenters(&self) -> Result<Vec<CloudDatacenter>, Error> {
        // There is a Linode API to return all regions, but for now the list is hard coded.
        let linode_regions = vec![
            "ap-northeast",
            "ap-south",
            "ap-southeast",
            "ap-west",
            "br-gru",
            "eu-central",
            "eu-west",
            "us-east",
            "us-iad",
            "us-sea",
        ];
        Ok(linode_regions
            .into_iter()
            .map(|r| CloudDatacenter::from_linode_region(r))
            .collect())
    }

    /// Return a list of cloud hosts.  Unfortunately, this does not contain script IDs.
    async fn list_hosts(&self) -> Result<Vec<(HostResourceId, IpAddr, Option<String>)>, Error> {
        let endpoint = format!("https://api.linode.com/v4/linode/instances");
        let request = self.client.get(&endpoint);
        let response = request.send().await.map_err(Self::map_error)?;
        let result = response.text().await.map_err(Self::map_error)?;
        let list: ListLinodeInstancesResponse = Self::parse_result(&result)?;
        let mut result: Vec<_> = vec![];
        for LinodeInstanceResponse {
            id: host_id,
            ipv4,
            record: LinodeInstance { label, .. },
            ..
        } in list.data.into_iter()
        {
            let ip_addr: IpAddr = ipv4
                .first()
                .and_then(|s| s.parse().ok())
                .ok_or(Error::Http(
                    StatusCode::FAILED_DEPENDENCY,
                    format!("{ipv4:?} does not contain an IP address"),
                ))?;
            result.push((
                HostResourceId(format!("{LINODE_PROVIDER_NAME}/{host_id}")),
                ip_addr,
                Some(label),
            ))
        }
        Ok(result)
    }

    fn provider_name(&self) -> &'static str {
        LINODE_PROVIDER_NAME
    }
}

const LINODE_IMAGE: &str = "linode/debian11";
const LINODE_PROVIDER_NAME: &str = "linode";
const LINODE_TYPE: &str = "g6-nanode-1";
const SWAP_SIZE_MB: usize = 128;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
struct LinodeInstance {
    authorized_keys: Option<Vec<String>>,
    image: String,
    label: String,
    tags: Vec<String>,
    region: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    root_pass: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stackscript_id: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    firewall_id: Option<usize>,
    #[serde(default)]
    private_ip: bool,
    #[serde(rename = "type")]
    linode_type: String,
    /// Note: not sent in responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    swap_size: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct LinodeInstanceResponse {
    id: usize,
    ipv4: Vec<String>,
    #[serde(flatten)]
    record: LinodeInstance,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
struct LinodeScript {
    description: Option<String>,
    images: Vec<String>,
    is_public: bool,
    label: String,
    rev_note: Option<String>,
    script: String,
}

#[derive(Debug, Deserialize)]
struct LinodeScriptResponse {
    id: usize,
    // username: String,
    // deployments_active: usize,
    #[serde(flatten)]
    record: LinodeScript,
}

#[derive(Debug, Deserialize)]
struct ListLinodeInstancesResponse {
    data: Vec<LinodeInstanceResponse>,
}

#[derive(Debug, Deserialize)]
struct ListLinodeScriptsResponse {
    data: Vec<LinodeScriptResponse>,
}
