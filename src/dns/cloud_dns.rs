// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{AwsDns, LinodeDns};
use crate::common::{CubConfig, Error};
use crate::datacenter::CloudDatacenter;
use crate::log::StringLogger;
use async_trait::async_trait;
use std::sync::Arc;
use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    net::IpAddr,
};

/// Cloud DNS trait
#[async_trait]
pub trait CloudDns {
    /// Read the DNS records of the specified domain (zone).
    async fn read_dns_records(&self, domain: &str) -> Result<DnsRecordSet, Error>;

    /// Update (or remove) the metadata of a particular host in the specified domain (zone).
    async fn update_dns_metadata(
        &self,
        domain: &str,
        hostname: &str,
        value: DnsRecord,
        ttl: Option<usize>,
    ) -> Result<String, Error>;

    /// Update multiple DNS records in the specified domain (zone).
    async fn update_dns_records(
        &self,
        domain: &str,
        record_set: DnsRecordSet,
    ) -> Result<String, Error> {
        let logger = StringLogger::default();
        // TODO: this could be optimized to avoid reading the domain multiple times.
        for (hostname, record) in record_set.metadata() {
            logger.trace(
                self.update_dns_metadata(domain, &hostname, record, None)
                    .await?,
            );
        }
        for (hostname, record) in record_set.routes() {
            logger.trace(
                self.update_dns_route(domain, &hostname, record, None)
                    .await?,
            );
        }
        Ok(logger.to_string())
    }

    /// Update (or remove) the route(s) to a particular host in the specified domain (zone).
    async fn update_dns_route(
        &self,
        domain: &str,
        hostname: &str,
        value: DnsRecord,
        ttl: Option<usize>,
    ) -> Result<String, Error>;
}

/// The types of DNS metadata and routes that are supported.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DnsRecord {
    /// The `A` record is for IP addresses with optional geographic region.
    A(HashMap<IpAddr, Option<CloudDatacenter>>),
    /// The `Cname` record is for aliases.
    Cname(String),
    /// The `Txt` record is for text.
    Txt(String),
    /// `None` is for clearing an existing route or metadata.
    None,
}

impl DnsRecord {
    /// Create a DNS record for a single IP address
    pub fn new_a(ip_addr: IpAddr) -> Self {
        let mut m = HashMap::new();
        m.insert(ip_addr, None);
        DnsRecord::A(m)
    }

    /// Create a DNS record for a single IP address in a datacenter
    pub fn new_ag(ip_addr: IpAddr, datacenter: CloudDatacenter) -> Self {
        let mut m = HashMap::new();
        m.insert(ip_addr, Some(datacenter));
        DnsRecord::A(m)
    }
}

impl Hash for DnsRecord {
    // The hash considers only the record type not the argument.
    fn hash<H: Hasher>(&self, state: &mut H) {
        let n = match self {
            DnsRecord::A(_) => 1,
            DnsRecord::Cname(_) => 2,
            DnsRecord::Txt(_) => 3,
            DnsRecord::None => 4,
        };
        n.hash(state);
    }
}

/// DNS record set for a domain.
#[derive(Default)]
pub struct DnsRecordSet(pub(crate) HashSet<(String, DnsRecord)>);

impl DnsRecordSet {
    /// Create a DNS record set builder.
    pub fn builder() -> DnsRecordSetBuilder {
        Default::default()
    }

    /// Returns the metadata records but not the route records.
    pub fn metadata(&self) -> HashMap<String, DnsRecord> {
        self.0
            .iter()
            .filter(|(_, record)| match record {
                DnsRecord::Txt(_) => true,
                _ => false,
            })
            .map(|(hostname, record)| (hostname.clone(), record.clone()))
            .collect()
    }

    /// Returns the route records but not the metadata records.
    pub fn routes(&self) -> HashMap<String, DnsRecord> {
        self.0
            .iter()
            .filter(|(_, record)| match record {
                DnsRecord::A(_) | DnsRecord::Cname(_) => true,
                _ => false,
            })
            .map(|(hostname, record)| (hostname.clone(), record.clone()))
            .collect()
    }
}

/// DNS record set builder.
#[derive(Default)]
pub struct DnsRecordSetBuilder {
    record_set: DnsRecordSet,
}

impl DnsRecordSetBuilder {
    /// Add an `A` record with a set of IP addresses.
    pub fn a(self, hostname: &str, ips: HashSet<IpAddr>) -> Self {
        let ipgeos = ips.into_iter().map(|ip| (ip, None)).collect();
        self.ag(hostname, ipgeos)
    }

    /// Add an `A` record with a set of IP addresses that have optional geographic region.
    pub fn ag(mut self, hostname: &str, ipgeos: HashMap<IpAddr, Option<CloudDatacenter>>) -> Self {
        self.record_set
            .0
            .insert((hostname.to_owned(), DnsRecord::A(ipgeos)));
        self
    }

    /// Complete building and then return the DNS record set.
    pub fn build(mut self) -> DnsRecordSet {
        DnsRecordSet(self.record_set.0.drain().collect())
    }

    /// The `Cname` record is for aliases.
    pub fn cname(mut self, hostname: &str, name: &str) -> Self {
        self.record_set
            .0
            .insert((hostname.to_owned(), DnsRecord::Cname(name.to_owned())));
        self
    }

    /// The `Txt` record is for text.
    pub fn txt(mut self, hostname: &str, text: &str) -> Self {
        self.record_set
            .0
            .insert((hostname.to_owned(), DnsRecord::Txt(text.to_owned())));
        self
    }
}

/// Cloud DNS client.
pub struct CloudDnsClient {
    aws: Arc<dyn CloudDns + Sync + Send>,
    linode: Arc<dyn CloudDns + Sync + Send>,
}

/// Multiple DNS APIs.
impl CloudDnsClient {
    /// Create a new cloud DNS client.
    pub async fn new(cub_config: &CubConfig) -> CloudDnsClient {
        Self {
            aws: Arc::new(AwsDns::new(cub_config).await),
            linode: Arc::new(LinodeDns::new(cub_config)),
        }
    }

    /// Choose which nameserver to use.
    pub async fn nameserver_api(
        &self,
        nameserver_api: Option<&str>,
    ) -> Arc<dyn CloudDns + Sync + Send> {
        match nameserver_api {
            Some("aws") => Arc::clone(&self.aws),
            _ => Arc::clone(&self.linode),
        }
    }
}
