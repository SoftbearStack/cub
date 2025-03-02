// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use super::{CloudDns, DnsRecord, DnsRecordSet};
use crate::common::{CubConfig, Error};
use crate::log::StringLogger;
use async_trait::async_trait;
use hyper::{http::HeaderValue, HeaderMap, StatusCode};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
    str::FromStr,
    time::Duration,
};

/// This struct implements `CloudDNS` for Linode.
pub struct LinodeDns {
    client: Client,
}

impl LinodeDns {
    const TIMEOUT_SECS: u64 = 5;
    const TTL_SECS: usize = 30;

    /// Create a `CloudDNS` for Linode.
    pub fn new(cub_config: &CubConfig) -> Self {
        #[derive(Deserialize)]
        struct LinodeConfig {
            personal_access_token: String,
        }
        #[derive(Deserialize)]
        struct ConfigToml {
            linode: LinodeConfig,
        }
        let ConfigToml {
            linode: LinodeConfig {
                personal_access_token,
            },
        } = cub_config.get().expect("linode.toml");

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
            client: Client::builder()
                .timeout(Duration::from_secs(Self::TIMEOUT_SECS))
                .default_headers(default_headers)
                .http1_only()
                .build()
                .unwrap(),
        }
    }

    async fn create_domain_record(
        &self,
        domain_id: usize,
        record: LinodeDomainRecord,
        logger: &StringLogger,
    ) -> Result<LinodeRecordResponse, Error> {
        logger.trace(format!(
            "domain {} hostname {} create {:?} record {}",
            domain_id, record.name, record.record_type, record.target
        ));
        let endpoint = format!("https://api.linode.com/v4/domains/{}/records", domain_id);
        let request = self.client.post(endpoint);
        let request = request.json(&record).build().map_err(Self::map_error)?;
        let diagnostic = format!("{record:?}");
        let response = self
            .client
            .execute(request)
            .await
            .map_err(Self::map_error)?;
        let text = response.text().await.map_err(Self::map_error)?;
        match serde_json::from_str(&text) {
            Ok(response) => Ok(response),
            Err(_) => {
                println!(">> {diagnostic}\n<< {text}");
                #[derive(Deserialize)]
                struct LinodeReason {
                    reason: String,
                }
                #[derive(Deserialize)]
                struct LinodeError {
                    errors: Vec<LinodeReason>,
                }
                match serde_json::from_str(&text) {
                    Ok(LinodeError { errors }) => {
                        let r = errors.into_iter().next().map(|r| r.reason);
                        Err(Error::Http(
                            StatusCode::FAILED_DEPENDENCY,
                            format!("linode error: {r:?}"),
                        ))
                    }
                    Err(_) => Err(Error::Http(
                        StatusCode::FAILED_DEPENDENCY,
                        format!("cannot parse linode error"),
                    )),
                }
            }
        }
    }

    async fn delete_domain_record(&self, domain_id: usize, id: usize) -> Result<(), Error> {
        let endpoint = format!(
            "https://api.linode.com/v4/domains/{}/records/{}",
            domain_id, id
        );
        let request = self
            .client
            .delete(endpoint)
            .build()
            .map_err(Self::map_error)?;
        self.client
            .execute(request)
            .await
            .map_err(Self::map_error)?;
        Ok(())
    }

    async fn get_domain_id(&self, domain_name: &str) -> Result<usize, Error> {
        self.list_linode_domains()
            .await?
            .data
            .iter()
            .find(|d| d.domain == domain_name)
            .map(|d| d.id)
            .ok_or(Error::Http(
                StatusCode::FAILED_DEPENDENCY,
                format!("Could not find domain {domain_name}"),
            ))
    }

    async fn list_linode_domains(&self) -> Result<ListLinodeDomainsResponse, Error> {
        let endpoint = "https://api.linode.com/v4/domains";
        let request = self.client.get(endpoint);
        let response = request.send().await.map_err(Self::map_error)?;
        response.json().await.map_err(Self::map_error)
    }

    async fn list_linode_records(
        &self,
        domain_id: usize,
    ) -> Result<ListLinodeRecordsReponse, Error> {
        let endpoint = format!("https://api.linode.com/v4/domains/{}/records", domain_id);
        let request = self.client.get(endpoint);
        let response = request.send().await.map_err(Self::map_error)?;
        response.json().await.map_err(Self::map_error)
    }

    fn map_error(e: reqwest::Error) -> Error {
        Error::Http(StatusCode::FAILED_DEPENDENCY, format!("{}", e))
    }

    fn parse_ip(target: &str, domain: &str, hostname: &str) -> Result<IpAddr, Error> {
        IpAddr::from_str(target).map_err(|_| {
            Error::Http(
                StatusCode::FAILED_DEPENDENCY,
                format!("Could not parse IP {target} of A record for hostname {hostname} in domain {domain}"),
            )
        })
    }

    async fn upsert_a_record(
        &self,
        domain: &str,
        domain_id: usize,
        hostname: &str,
        ttl_sec: usize,
        id_records: &Vec<&LinodeRecordResponse>,
        ip_addrs: HashSet<IpAddr>,
        logger: &StringLogger,
    ) -> Result<(), Error> {
        let mut removals: Vec<usize> = Vec::new();
        let mut found: HashSet<IpAddr> = HashSet::new();
        for LinodeRecordResponse {
            id,
            record:
                LinodeDomainRecord {
                    record_type,
                    target,
                    ..
                },
        } in id_records.iter()
        {
            match record_type {
                LinodeRecordType::A => {
                    let ip = Self::parse_ip(&target, domain, hostname)?;
                    if !(ip_addrs.contains(&ip) && found.insert(ip)) {
                        removals.push(*id);
                    }
                }
                LinodeRecordType::Cname => {
                    removals.push(*id);
                }
                _ => {
                    // Ignore TXT records, etc.
                }
            }
        }

        let mut adds: Vec<LinodeDomainRecord> = Vec::new();
        for ip in ip_addrs {
            if !found.contains(&ip) {
                adds.push(LinodeDomainRecord {
                    name: hostname.to_string(),
                    record_type: LinodeRecordType::A,
                    target: ip.to_string(),
                    ttl_sec,
                });
            }
        }

        for record_id in removals {
            self.delete_domain_record(domain_id, record_id).await?;
        }

        for record in adds {
            self.create_domain_record(domain_id, record, &logger)
                .await?;
        }

        Ok(())
    }
}

#[async_trait]
impl CloudDns for LinodeDns {
    /// Read DNS record set for the specified domain (zone).
    async fn read_dns_records(&self, domain: &str) -> Result<DnsRecordSet, Error> {
        let domain_id = self.get_domain_id(domain).await?;

        let list: ListLinodeRecordsReponse = self.list_linode_records(domain_id).await?;
        let list_len = list.data.len();

        let mut a_ips: HashMap<String, HashSet<IpAddr>> = HashMap::new();
        let mut other: HashMap<String, DnsRecord> = HashMap::new();

        for LinodeRecordResponse {
            record:
                LinodeDomainRecord {
                    name: hostname,
                    target,
                    record_type,
                    ..
                },
            ..
        } in list.data.into_iter()
        {
            match record_type {
                LinodeRecordType::A => {
                    let ip = Self::parse_ip(&target, &domain, &hostname)?;
                    a_ips.entry(hostname).or_insert(HashSet::new()).insert(ip);
                }
                LinodeRecordType::Cname => {
                    other.insert(hostname, DnsRecord::Cname(target));
                }
                LinodeRecordType::Txt => {
                    other.insert(hostname, DnsRecord::Txt(target));
                }
                _ => {}
            }
        }

        // May be more capacity than required, but always enough.
        let mut dns_records = HashSet::with_capacity(list_len);

        for (hostname, record) in other.into_iter() {
            dns_records.insert((hostname, record));
        }

        for (hostname, ips) in a_ips.into_iter() {
            let ipgeos: HashMap<_, _> = ips.into_iter().map(|ip| (ip, None)).collect();
            dns_records.insert((hostname, DnsRecord::A(ipgeos)));
        }

        Ok(DnsRecordSet(dns_records))
    }

    async fn update_dns_metadata(
        &self,
        domain: &str,
        hostname: &str,
        value: DnsRecord,
        ttl: Option<usize>,
    ) -> Result<String, Error> {
        let logger = StringLogger::default();
        let domain_id = self.get_domain_id(domain).await?;

        let ttl_sec = if let Some(ttl) = ttl {
            if ttl == 0 {
                Self::TTL_SECS
            } else {
                ttl
            }
        } else {
            Self::TTL_SECS
        };

        let response = self.list_linode_records(domain_id).await?;

        let id_records: Vec<_> = response
            .data
            .iter()
            .filter(|r| {
                r.record.name == hostname
                    && match r.record.record_type {
                        LinodeRecordType::Txt | LinodeRecordType::Mx | LinodeRecordType::Srv => {
                            true
                        }
                        _ => false,
                    }
            })
            .collect();

        match value {
            DnsRecord::Txt(text) => {
                for record_id in id_records
                    .iter()
                    .filter(|r| r.record.record_type == LinodeRecordType::Txt)
                    .map(|r| r.id)
                {
                    self.delete_domain_record(domain_id, record_id).await?;
                }
                self.create_domain_record(
                    domain_id,
                    LinodeDomainRecord {
                        name: hostname.to_string(),
                        record_type: LinodeRecordType::Txt,
                        target: text,
                        ttl_sec,
                    },
                    &logger,
                )
                .await?;
            }
            DnsRecord::None => {
                for record_id in id_records
                    .iter()
                    .filter(|r| r.record.record_type == LinodeRecordType::Txt)
                    .map(|r| r.id)
                {
                    self.delete_domain_record(domain_id, record_id).await?;
                }
            }
            _ => logger.trace("non-metadata record ignored".to_string()),
        }
        Ok(logger.to_string())
    }

    async fn update_dns_route(
        &self,
        domain: &str,
        hostname: &str,
        value: DnsRecord,
        ttl: Option<usize>,
    ) -> Result<String, Error> {
        let logger = StringLogger::default();
        let domain_id = self.get_domain_id(domain).await?;

        let ttl_sec = if let Some(ttl) = ttl {
            if ttl == 0 {
                Self::TTL_SECS
            } else {
                ttl
            }
        } else {
            Self::TTL_SECS
        };
        let response = self.list_linode_records(domain_id).await?;

        let id_records: Vec<_> = response
            .data
            .iter()
            .filter(|r| r.record.name == hostname)
            .collect();

        match value {
            DnsRecord::A(ipgeos) => {
                // For now, Linode ignores regions.
                let mut ip_addrs = HashSet::new();
                for ip in ipgeos.keys() {
                    ip_addrs.insert(*ip);
                }
                self.upsert_a_record(
                    domain,
                    domain_id,
                    hostname,
                    ttl_sec,
                    &id_records,
                    ip_addrs,
                    &logger,
                )
                .await?
            }
            DnsRecord::Cname(link) => {
                let mut removals: Vec<usize> = Vec::new();
                let mut found: bool = false;
                for LinodeRecordResponse {
                    id,
                    record:
                        LinodeDomainRecord {
                            record_type,
                            target,
                            ..
                        },
                } in id_records.iter()
                {
                    match record_type {
                        LinodeRecordType::A => {
                            removals.push(*id);
                        }
                        LinodeRecordType::Cname => {
                            if !found && *target == link {
                                found = true;
                            } else {
                                removals.push(*id);
                            }
                        }
                        _ => {
                            // Ignore TXT records, etc.
                        }
                    }
                }

                for record_id in removals {
                    self.delete_domain_record(domain_id, record_id).await?;
                }

                if !found {
                    self.create_domain_record(
                        domain_id,
                        LinodeDomainRecord {
                            name: hostname.to_string(),
                            record_type: LinodeRecordType::Cname,
                            target: link,
                            ttl_sec,
                        },
                        &logger,
                    )
                    .await?;
                }
            }
            DnsRecord::None => {
                for LinodeRecordResponse {
                    id: record_id,
                    record: LinodeDomainRecord { record_type, .. },
                    ..
                } in id_records.iter()
                {
                    match record_type {
                        LinodeRecordType::A | LinodeRecordType::Cname => {
                            self.delete_domain_record(domain_id, *record_id).await?;
                        }
                        _ => {
                            // Ignore TXT records, etc.
                        }
                    }
                }
            }
            _ => logger.trace("non route record ignored".to_string()),
        }

        Ok(logger.to_string())
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
struct LinodeDomainRecord {
    name: String,
    target: String,
    ttl_sec: usize,
    #[serde(rename = "type")]
    record_type: LinodeRecordType,
}

#[derive(Debug, Deserialize)]
struct LinodeRecordResponse {
    id: usize,
    #[serde(flatten)]
    record: LinodeDomainRecord,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum LinodeRecordType {
    A,
    Aaaa,
    Ns,
    Mx,
    Cname,
    Txt,
    Srv,
    Caa,
    Ptr,
}

#[derive(Debug, Deserialize)]
struct LinodeDomainResponse {
    id: usize,
    domain: String,
}

#[derive(Debug, Deserialize)]
struct ListLinodeDomainsResponse {
    data: Vec<LinodeDomainResponse>,
}

#[derive(Debug, Deserialize)]
struct ListLinodeRecordsReponse {
    data: Vec<LinodeRecordResponse>,
}
