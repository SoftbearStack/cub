// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use super::{CloudDns, DnsRecord, DnsRecordSet};
use crate::aws::load_aws_config;
use crate::common::{CubConfig, Error};
use crate::datacenter::CloudDatacenter;
use crate::impl_wrapper_str;
use crate::log::StringLogger;
use async_trait::async_trait;
use aws_sdk_dynamodb::error::BuildError;
use aws_sdk_route53::types::{
    Change, ChangeAction, ChangeBatch, GeoProximityLocation, ResourceRecord, ResourceRecordSet,
    RrType,
};
use aws_sdk_route53::Client;
use hyper::StatusCode;
use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
    str::FromStr,
};

const DEBUG: bool = false;

/// This struct implements `CloudDNS` for Aws.
pub struct AwsDns {
    client: Client,
}

impl AwsDns {
    const TTL_SECS: usize = 30;

    /// Create a `CloudDNS` for AWS.
    pub async fn new(cub_config: &CubConfig) -> Self {
        let aws_config = load_aws_config(cub_config).await;
        let client = Client::new(&aws_config);
        Self { client }
    }

    async fn create_domain_record(
        &self,
        domain_id: &AwsDomainId,
        record: ExtendedDnsRecord,
        logger: &StringLogger,
    ) -> Result<(), Error> {
        let ExtendedDnsRecord {
            datacenter,
            name,
            record_type,
            targets,
            ttl_sec,
            ..
        } = record;
        logger.trace(format!(
            "domain {domain_id} hostname {name} create {record_type:?} record {targets:?}",
        ));
        let geo_proximity_location = datacenter.as_ref().map(|dc| {
            GeoProximityLocation::builder()
                .aws_region(dc.nearest_aws_region().to_owned())
                .build()
        });
        let set_identifier = datacenter.map(|dc| dc.nearest_aws_region().to_owned());
        let rrs_builder = if let (Some(geo_proximity_location), Some(set_identifier)) =
            (geo_proximity_location, set_identifier)
        {
            ResourceRecordSet::builder()
                .geo_proximity_location(geo_proximity_location)
                .set_identifier(set_identifier)
        } else {
            ResourceRecordSet::builder()
        };
        let batch = ChangeBatch::builder()
            .changes(
                Change::builder()
                    .action(ChangeAction::Upsert)
                    .resource_record_set(
                        rrs_builder
                            .name(name)
                            .r#type(record_type)
                            .ttl(ttl_sec as i64)
                            .set_resource_records(Some(
                                targets
                                    .into_iter()
                                    .map(|target| {
                                        ResourceRecord::builder()
                                            .value(target)
                                            .build()
                                            .map_err(Self::map_build_err)
                                    })
                                    .collect::<Result<Vec<_>, _>>()?,
                            ))
                            .build()
                            .map_err(Self::map_build_err)?,
                    )
                    .build()
                    .map_err(Self::map_build_err)?,
            )
            .build()
            .map_err(Self::map_build_err)?;
        let _ = self
            .client
            .change_resource_record_sets()
            .hosted_zone_id(domain_id.to_string())
            .change_batch(batch)
            .send()
            .await
            .map_err(|e| {
                Error::Anyhow(
                    e.into(),
                    format!("create_domain_record: cannot change records"),
                )
            })?;
        Ok(())
    }

    async fn delete_domain_record(
        &self,
        domain_id: &AwsDomainId,
        record_id: &AwsRecordId,
    ) -> Result<(), Error> {
        let batch = ChangeBatch::builder()
            .changes(
                Change::builder()
                    .action(ChangeAction::Delete)
                    .resource_record_set(record_id.0.clone())
                    .build()
                    .map_err(Self::map_build_err)?,
            )
            .build()
            .map_err(Self::map_build_err)?;
        let _ = self
            .client
            .change_resource_record_sets()
            .hosted_zone_id(domain_id.to_string())
            .change_batch(batch)
            .send()
            .await
            .map_err(|e| {
                Error::Anyhow(
                    e.into(),
                    format!("delete_domain_record(domain={domain_id}, record={record_id:?})"),
                )
            })?;
        Ok(())
    }

    fn double_quoted(text: &str) -> String {
        if text.starts_with("\"") && text.ends_with("\"") {
            text.to_string()
        } else {
            format!("\"{text}\"")
        }
    }

    fn fully_qualified(hostname: &str, domain: &str) -> String {
        if hostname.len() == 0 {
            domain.to_string()
        } else if hostname.ends_with(domain) {
            hostname.to_string()
        } else {
            format!("{hostname}.{domain}")
        }
    }

    async fn get_domain_id(&self, domain_name: &str) -> Result<AwsDomainId, Error> {
        Ok(self
            .list_route53_domains()
            .await?
            .into_iter()
            .find(|(_, d)| *d == domain_name)
            .map(|(id, _)| id)
            .ok_or(Error::Http(
                StatusCode::FAILED_DEPENDENCY,
                format!("Could not find domain {domain_name}"),
            ))?)
    }

    async fn list_route53_domains(&self) -> Result<Vec<(AwsDomainId, String)>, Error> {
        let output = self
            .client
            .list_hosted_zones()
            .send()
            .await
            .map_err(|e| Error::Anyhow(e.into(), format!("list_route53_domains()")))?;
        if output.is_truncated() {
            return Err(Error::Http(
                StatusCode::FAILED_DEPENDENCY,
                format!("DNS result truncated"),
            ));
        }
        Ok(output
            .hosted_zones()
            .into_iter()
            .map(|hz| (hz.id(), hz.name()))
            .map(|(id, name)| (AwsDomainId(id.to_string()), Self::sans_trailing_dot(name)))
            .collect())
    }

    async fn list_route53_records(
        &self,
        domain_id: &AwsDomainId,
    ) -> Result<Vec<(AwsRecordId, ExtendedDnsRecord)>, Error> {
        let output = self
            .client
            .list_resource_record_sets()
            .hosted_zone_id(domain_id.to_string())
            .send()
            .await
            .map_err(|e| {
                Error::Anyhow(
                    e.into(),
                    format!("list_route53_records(domain={domain_id})"),
                )
            })?;
        if output.is_truncated() {
            return Err(Error::Http(
                StatusCode::FAILED_DEPENDENCY,
                format!("{domain_id}: DNS result truncated"),
            ));
        }
        Ok(output
            .resource_record_sets()
            .into_iter()
            .map(|rrs| {
                if DEBUG {
                    println!("DNS Record: {rrs:?}");
                }
                rrs
            })
            // For now, A records must have an IP address or they will be ignored.
            .filter(|rrs| rrs.alias_target().is_none())
            .map(|rrs| {
                (
                    AwsRecordId(rrs.clone()),
                    ExtendedDnsRecord {
                        name: Self::parse_name(rrs.name()),
                        datacenter: rrs.geo_proximity_location().and_then(|gpl| {
                            gpl.aws_region()
                                .map(|aws_region| CloudDatacenter::from_aws_region(aws_region))
                        }),
                        targets: rrs
                            .resource_records()
                            .iter()
                            .map(|rr| rr.value().to_owned())
                            .collect::<Vec<_>>(),
                        ttl_sec: rrs.ttl().map(|ttl| ttl as usize).unwrap_or(Self::TTL_SECS),
                        record_type: rrs.r#type().clone(),
                    },
                )
            })
            .collect())
    }

    fn map_build_err(e: BuildError) -> Error {
        Error::Anyhow(e.into(), format!("cannot build container"))
    }

    fn parse_ip(target: &str, domain: &str, hostname: &str) -> Result<IpAddr, Error> {
        IpAddr::from_str(target).map_err(|_| {
            Error::Http(
                StatusCode::FAILED_DEPENDENCY,
                format!("Could not parse IP {target} of A record for hostname {hostname} in domain {domain}"),
            )
        })
    }

    fn parse_name(target: &str) -> String {
        Self::sans_trailing_dot(&target.replace("\\052", "*"))
    }

    fn sans_domain(domain: &str, hostname: &str) -> String {
        if domain == hostname {
            "".to_string()
        } else {
            let dotdomain = format!(".{domain}");
            if hostname.ends_with(&dotdomain) {
                hostname[..hostname.len() - domain.len() - 1].to_string()
            } else {
                // Never happens.
                hostname.to_string()
            }
        }
    }

    fn sans_trailing_dot(name: &str) -> String {
        if name.ends_with(".") {
            name[..name.len() - 1].to_string()
        } else {
            name.to_string()
        }
    }

    async fn upsert_a_record(
        &self,
        domain: &str,
        domain_id: AwsDomainId,
        fq_hostname: &str,
        ttl_sec: usize,
        id_records: &Vec<(AwsRecordId, ExtendedDnsRecord)>,
        ipgeos: HashMap<IpAddr, Option<CloudDatacenter>>,
        logger: &StringLogger,
    ) -> Result<(), Error> {
        let mut removals: Vec<AwsRecordId> = Vec::new();
        let mut found: HashSet<IpAddr> = HashSet::new();
        for (
            id,
            ExtendedDnsRecord {
                record_type,
                targets,
                datacenter,
                ..
            },
        ) in id_records.iter()
        {
            match record_type {
                RrType::A => {
                    let ips = targets
                        .iter()
                        .map(|target| Self::parse_ip(&target, &domain, &fq_hostname))
                        .collect::<Result<Vec<_>, _>>()?;
                    // AWS supports one A record per Option<CloudDatacenter>. If the record isn't
                    // exactly right, must remove it.
                    // TODO: if the new TTL doesn't match the previous TTL, re-create the record.
                    if ips.iter().all(|ip| {
                        ipgeos.get(ip).is_some_and(|dc| dc == datacenter) && !found.contains(ip)
                    }) && ipgeos
                        .iter()
                        .filter(|(_, geo)| *geo == datacenter)
                        .all(|(ip, _)| ips.contains(ip))
                    {
                        found.extend(ips);
                    } else {
                        removals.push(id.clone());
                    }
                }
                RrType::Cname => {
                    removals.push(id.clone());
                }
                _ => {
                    // Ignore TXT records, etc.
                }
            }
        }

        let mut adds: HashMap<Option<CloudDatacenter>, ExtendedDnsRecord> = HashMap::new();

        for (ip, datacenter) in ipgeos {
            if found.contains(&ip) {
                continue;
            }
            let record = adds
                .entry(datacenter.clone())
                .or_insert_with(|| ExtendedDnsRecord {
                    datacenter,
                    name: fq_hostname.to_owned(),
                    record_type: RrType::A,
                    targets: Vec::new(),
                    ttl_sec,
                });
            record.targets.push(ip.to_string());
        }

        for record_id in removals {
            self.delete_domain_record(&domain_id, &record_id).await?;
        }

        // TODO: can set these in a single command.
        for record in adds.into_values() {
            self.create_domain_record(&domain_id, record, &logger)
                .await?;
        }

        Ok(())
    }
}

#[async_trait]
impl CloudDns for AwsDns {
    /// Read DNS record set for the specified domain (zone).
    async fn read_dns_records(&self, domain: &str) -> Result<DnsRecordSet, Error> {
        let domain_id = self.get_domain_id(domain).await?;
        if DEBUG {
            println!("domain_id={domain_id}");
        }

        let list: Vec<(AwsRecordId, ExtendedDnsRecord)> =
            self.list_route53_records(&domain_id).await?;
        let list_len = list.len();

        let mut a_ipgeos: HashMap<String, HashMap<IpAddr, Option<CloudDatacenter>>> =
            HashMap::new();
        let mut other: HashMap<String, DnsRecord> = HashMap::new();

        for (
            _,
            ExtendedDnsRecord {
                datacenter,
                name: hostname,
                targets,
                record_type,
                ..
            },
        ) in list.into_iter()
        {
            match record_type {
                RrType::A => {
                    let ips = targets
                        .into_iter()
                        .map(|target| Self::parse_ip(&target, &domain, &hostname))
                        .collect::<Result<Vec<_>, _>>()?;
                    let entry = a_ipgeos.entry(hostname).or_insert(HashMap::new());
                    for ip in ips {
                        entry.insert(ip, datacenter.clone());
                    }
                }
                RrType::Cname => {
                    if targets.len() == 1 {
                        other.insert(
                            hostname,
                            DnsRecord::Cname(targets.into_iter().next().unwrap()),
                        );
                    }
                }
                RrType::Txt => {
                    // TODO: Support multiple TXT
                    if targets.len() == 1 {
                        other.insert(
                            hostname,
                            DnsRecord::Txt(targets.into_iter().next().unwrap()),
                        );
                    }
                }
                _ => {}
            }
        }

        // May be more capacity than required, but always enough.
        let mut dns_records = HashSet::with_capacity(list_len);

        for (hostname, record) in other.into_iter() {
            dns_records.insert((Self::sans_domain(domain, &hostname), record));
        }

        for (hostname, ip_geo_opts) in a_ipgeos.into_iter() {
            dns_records.insert((
                Self::sans_domain(domain, &hostname),
                DnsRecord::A(ip_geo_opts),
            ));
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
        let fq_hostname = Self::fully_qualified(hostname, domain);

        let ttl_sec = if let Some(ttl) = ttl {
            if ttl == 0 {
                Self::TTL_SECS
            } else {
                ttl
            }
        } else {
            Self::TTL_SECS
        };

        let response = self.list_route53_records(&domain_id).await?;
        let id_records: Vec<_> = response
            .into_iter()
            .filter(
                |(
                    _,
                    ExtendedDnsRecord {
                        name, record_type, ..
                    },
                )| {
                    *name == fq_hostname
                        && match record_type {
                            RrType::Txt | RrType::Mx | RrType::Srv => true,
                            _ => false,
                        }
                },
            )
            .collect();

        match value {
            DnsRecord::Txt(text) => {
                for (record_id, _, _) in id_records
                    .iter()
                    .filter(|(_, ExtendedDnsRecord { record_type, .. })| {
                        *record_type == RrType::Txt
                    })
                    .map(
                        |(
                            record_id,
                            ExtendedDnsRecord {
                                name, record_type, ..
                            },
                        )| (record_id, name, record_type),
                    )
                {
                    self.delete_domain_record(&domain_id, record_id).await?;
                }
                self.create_domain_record(
                    &domain_id,
                    ExtendedDnsRecord {
                        datacenter: None,
                        name: fq_hostname,
                        record_type: RrType::Txt,
                        targets: vec![Self::double_quoted(&text)],
                        ttl_sec,
                    },
                    &logger,
                )
                .await?;
            }
            DnsRecord::None => {
                for (record_id, _, _) in id_records
                    .iter()
                    .filter(|(_, ExtendedDnsRecord { record_type, .. })| {
                        *record_type == RrType::Txt
                    })
                    .map(
                        |(
                            record_id,
                            ExtendedDnsRecord {
                                name, record_type, ..
                            },
                        )| (record_id, name, record_type),
                    )
                {
                    self.delete_domain_record(&domain_id, record_id).await?;
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
        let fq_hostname = Self::fully_qualified(hostname, domain);

        let ttl_sec = if let Some(ttl) = ttl {
            if ttl == 0 {
                Self::TTL_SECS
            } else {
                ttl
            }
        } else {
            Self::TTL_SECS
        };
        let response = self.list_route53_records(&domain_id).await?;
        let id_records: Vec<_> = response
            .into_iter()
            .filter(|(_, ExtendedDnsRecord { name, .. })| *name == fq_hostname)
            .collect();

        match value {
            DnsRecord::A(ipgeos) => {
                self.upsert_a_record(
                    domain,
                    domain_id,
                    &fq_hostname,
                    ttl_sec,
                    &id_records,
                    ipgeos,
                    &logger,
                )
                .await?
            }
            DnsRecord::Cname(link) => {
                let mut removals: Vec<AwsRecordId> = Vec::new();
                let mut found: bool = false;
                for (
                    id,
                    ExtendedDnsRecord {
                        record_type,
                        targets,
                        ..
                    },
                ) in id_records.iter()
                {
                    match record_type {
                        RrType::A => {
                            removals.push(id.clone());
                        }
                        RrType::Cname => {
                            if !found && targets.len() == 1 && targets[0] == link {
                                found = true;
                            } else {
                                removals.push(id.clone());
                            }
                        }
                        _ => {
                            // Ignore TXT records, etc.
                        }
                    }
                }

                for record_id in removals {
                    self.delete_domain_record(&domain_id, &record_id).await?;
                }

                if !found {
                    self.create_domain_record(
                        &domain_id,
                        ExtendedDnsRecord {
                            datacenter: None,
                            name: fq_hostname,
                            record_type: RrType::Cname,
                            targets: vec![link],
                            ttl_sec,
                        },
                        &logger,
                    )
                    .await?;
                }
            }
            DnsRecord::None => {
                for (record_id, ExtendedDnsRecord { record_type, .. }) in id_records.iter() {
                    match record_type {
                        RrType::A | RrType::Cname => {
                            self.delete_domain_record(&domain_id, record_id).await?;
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

/// AwsDomainId
#[derive(Clone)]
pub struct AwsDomainId(pub String);
impl_wrapper_str!(AwsDomainId);

/// AWS record ID
#[derive(Clone, Debug)]
pub struct AwsRecordId(ResourceRecordSet);

#[derive(Debug, Eq, PartialEq)]
struct ExtendedDnsRecord {
    datacenter: Option<CloudDatacenter>,
    name: String,
    targets: Vec<String>,
    ttl_sec: usize,
    record_type: RrType,
}
