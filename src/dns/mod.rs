// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

/// Support for AWS Route 53.
mod aws;
/// Cloud DNS trait
mod cloud_dns;
/// Support for Linode (aka Akami)
mod linode;
/// Unit tests
mod tests;

pub use self::aws::AwsDns;
pub use self::cloud_dns::{CloudDns, CloudDnsClient, DnsRecord, DnsRecordSet, DnsRecordSetBuilder};
pub use self::linode::LinodeDns;
