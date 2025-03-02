// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use super::LinodeHosts;
use crate::common::{CubConfig, Error};
use crate::datacenter::CloudDatacenter;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;

/// Host resource ID. For example, the ID of a virtual host.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HostResourceId(pub String);
crate::impl_wrapper_str!(HostResourceId);

/// Host parameters.
pub struct HostParameters(pub HashMap<String, String>);

/// Cloud hosts
#[async_trait]
pub trait CloudHosts {
    /// Allocate a new virtual host.
    async fn create_host(
        &self,
        label: &str,
        group: Option<&str>,
        hostname: &str,
        datacenter: CloudDatacenter,
        script: &str,
        parameters: Option<HostParameters>,
    ) -> Result<(HostResourceId, IpAddr), Error>;

    /// Delete virtual host.
    async fn delete_host(&self, id: &HostResourceId) -> Result<(), Error>;

    /// List datacenters.
    async fn list_datacenters(&self) -> Result<Vec<CloudDatacenter>, Error>;

    /// List virtual hosts.
    async fn list_hosts(&self) -> Result<Vec<(HostResourceId, IpAddr, Option<String>)>, Error>;

    /// Provider name.  For example, "linode".
    fn provider_name(&self) -> &'static str;
}

/// Cloud hosts client.
pub struct CloudHostsClient {
    linode: Arc<dyn CloudHosts + Sync + Send>,
}

/// Multiple DNS APIs.
impl CloudHostsClient {
    /// Create a new cloud hosts client.
    pub async fn new(cub_config: &CubConfig) -> CloudHostsClient {
        Self {
            linode: Arc::new(LinodeHosts::new(cub_config)),
        }
    }

    /// Allocate a new virtual host.
    pub async fn create_host(
        &self,
        label: &str,
        group: Option<&str>,
        hostname: &str,
        datacenter: CloudDatacenter,
        script: &str,
        parameters: Option<HostParameters>,
    ) -> Result<(HostResourceId, IpAddr), Error> {
        self.linode
            .create_host(label, group, hostname, datacenter, script, parameters)
            .await
    }

    /// Delete virtual host.
    pub async fn delete_host(&self, id: &HostResourceId) -> Result<(), Error> {
        self.linode.delete_host(id).await
    }

    /// List datacenters.
    pub async fn list_datacenters(&self) -> Result<Vec<CloudDatacenter>, Error> {
        self.linode.list_datacenters().await
    }

    /// List hosts.
    pub async fn list_hosts(&self) -> Result<Vec<(HostResourceId, IpAddr, Option<String>)>, Error> {
        self.linode.list_hosts().await
    }

    /// Choose which provider to use.
    pub async fn provider_name(
        &self,
        provider_name: Option<&str>,
    ) -> Arc<dyn CloudHosts + Sync + Send> {
        match provider_name {
            _ => Arc::clone(&self.linode),
        }
    }
}
