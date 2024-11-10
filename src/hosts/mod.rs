// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

/// Cloud host trait
mod cloud_hosts;
/// Support for Linode (aka Akami)
mod linode;
/// Unit tests
mod tests;

pub use self::cloud_hosts::{CloudHosts, CloudHostsClient, HostParameters, HostResourceId};
pub use self::linode::LinodeHosts;
