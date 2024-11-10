// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::common::Error;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

// Datacenter city.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DcCity {
    Boardman,
    Frankfurt,
    London,
    Mumbai,
    Newark,
    Nuremberg,
    Washington,
    Singapore,
    SaoPaulo,
    Seattle,
    Sydney,
    Tokyo,
}

impl DcCity {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Boardman => "Boardman",
            Self::Frankfurt => "Frankfurt",
            Self::London => "London",
            Self::Mumbai => "Mumbai",
            Self::Newark => "Newark",
            Self::Nuremberg => "Nuremberg",
            Self::Washington => "Washington DC",
            Self::Singapore => "Singapore",
            Self::SaoPaulo => "Sao Paulo",
            Self::Seattle => "Seattle",
            Self::Sydney => "Sydney",
            Self::Tokyo => "Tokyo",
        }
    }

    // Returns city corresponding to the specified AWS region.
    #[cfg(feature = "aws")]
    fn from_aws_region(label: &str) -> Option<Self> {
        match label {
            "ap-northeast-1" => Some(Self::Tokyo),
            "ap-south-1" => Some(Self::Mumbai),
            "ap-southeast-1" => Some(Self::Singapore),
            "ap-southeast-2" => Some(Self::Sydney),
            "eu-central-1" => Some(Self::Frankfurt),
            "eu-west-2" => Some(Self::London),
            "sa-east-1" => Some(Self::SaoPaulo),
            "us-east-1" => Some(Self::Washington),
            "us-west-2" => Some(Self::Boardman),
            _ => None,
        }
    }

    // Returns city corresponding to the specified Hetzner region.
    #[cfg(feature = "hetzner")]
    fn from_hetzner_region(label: &str) -> Option<Self> {
        match label {
            // Ashburn, Virginia, is a suburb of DC.
            "ash" => Some(Self::Washington),
            "nbg1" => Some(Self::Nuremberg),
            _ => None,
        }
    }

    // Returns city corresponding to the specified Linode region.
    #[cfg(feature = "linode")]
    fn from_linode_region(label: &str) -> Option<Self> {
        match label {
            "eu-central" => Some(Self::Frankfurt),
            "eu-west" => Some(Self::London),
            "ap-west" => Some(Self::Mumbai),
            "us-east" => Some(Self::Newark),
            "us-iad" => Some(Self::Washington),
            "ap-south" => Some(Self::Singapore),
            "br-gru" => Some(Self::SaoPaulo),
            "us-sea" => Some(Self::Seattle),
            "ap-southeast" => Some(Self::Sydney),
            "ap-northeast" => Some(Self::Tokyo),
            _ => None,
        }
    }

    fn from_region(label: &str, provider: DcProvider) -> Option<Self> {
        match provider {
            #[cfg(feature = "aws")]
            DcProvider::Aws => Self::from_aws_region(label),
            #[cfg(feature = "hetzner")]
            DcProvider::Hetzner => Self::from_hetzner_region(label),
            #[cfg(feature = "linode")]
            DcProvider::Linode => Self::from_linode_region(label),
            _ => None,
        }
    }

    /// Returns the AWS region corresponding to `DcCity` if any.
    #[cfg(feature = "aws")]
    pub fn to_aws_region(&self) -> Option<&'static str> {
        match self {
            Self::Boardman => Some("us-west-2"),
            Self::Frankfurt => Some("eu-central-1"),
            Self::Mumbai => Some("ap-south-1"),
            Self::London => Some("eu-west-2"),
            Self::SaoPaulo => Some("sa-east-1"),
            Self::Singapore => Some("ap-southeast-1"),
            Self::Sydney => Some("ap-southeast-2"),
            Self::Tokyo => Some("ap-northeast-1"),
            // Ashburn, Virginia, is a suburb of DC.
            Self::Washington => Some("us-east-1"),
            _ => None,
        }
    }

    /// Returns the Hetzner region corresponding to `DcCity` if any.
    #[cfg(feature = "hetzner")]
    fn to_hetzner_region(&self) -> Option<&'static str> {
        match self {
            Self::Washington => Some("ash"),
            Self::Nuremberg => Some("nbg1"),
            _ => None,
        }
    }

    /// Returns the Linode region corresponding to `DcCity` if any.
    #[cfg(feature = "linode")]
    fn to_linode_region(&self) -> Option<&'static str> {
        match self {
            Self::Frankfurt => Some("eu-central"),
            Self::London => Some("eu-west"),
            Self::Mumbai => Some("ap-west"),
            Self::Newark => Some("us-east"),
            Self::Washington => Some("us-iad"),
            Self::Singapore => Some("ap-south"),
            Self::SaoPaulo => Some("br-gru"),
            Self::Seattle => Some("us-sea"),
            Self::Sydney => Some("ap-southeast"),
            Self::Tokyo => Some("ap-northeast"),
            _ => None,
        }
    }

    fn to_region(&self, provider: DcProvider) -> Option<&'static str> {
        match provider {
            #[cfg(feature = "aws")]
            DcProvider::Aws => self.to_aws_region(),
            #[cfg(feature = "hetzner")]
            DcProvider::Hetzner => self.to_hetzner_region(),
            #[cfg(feature = "linode")]
            DcProvider::Linode => self.to_linode_region(),
            _ => None,
        }
    }
}

impl Display for DcCity {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(self.as_str())
    }
}

impl FromStr for DcCity {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_ref() {
            "Tokyo" => Ok(DcCity::Tokyo),
            "Mumbai" => Ok(DcCity::Mumbai),
            "Frankfurt" => Ok(DcCity::Frankfurt),
            "London" => Ok(DcCity::London),
            "Singapore" => Ok(DcCity::Singapore),
            "Sydney" => Ok(DcCity::Sydney),
            "Sao Paulo" => Ok(DcCity::SaoPaulo),
            "Washington DC" => Ok(DcCity::Washington),
            "Seattle" => Ok(DcCity::Seattle),
            _ => Err(()),
        }
    }
}

// Datacenter provider.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum DcProvider {
    Aws,
    #[default]
    Developer,
    Hetzner,
    Linode,
}

impl DcProvider {
    pub fn as_display_name(&self) -> &'static str {
        match self {
            Self::Aws => "AWS",
            Self::Developer => "Developer",
            Self::Hetzner => "Hetzner",
            Self::Linode => "Linode",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Aws => "aws",
            Self::Developer => "developer",
            Self::Hetzner => "hetzner",
            Self::Linode => "linode",
        }
    }
}

impl Display for DcProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(self.as_str())
    }
}

impl FromStr for DcProvider {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_ref() {
            "aws" => Ok(DcProvider::Aws),
            "developer" => Ok(DcProvider::Developer),
            "hetzner" => Ok(DcProvider::Hetzner),
            "linode" => Ok(DcProvider::Linode),
            _ => Err(()),
        }
    }
}

/// Cloud datacenter provider and location. For example, "Linode/eu-central".
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct CloudDatacenter(String);

impl CloudDatacenter {
    /// Returns city where the `CloudDatacenter` is located.
    fn city(&self) -> Option<(DcProvider, DcCity)> {
        self.0
            .split_once("/")
            .map(|(provider_name, region)| (provider_name.parse().ok(), region))
            .filter(|(provider, _)| provider.is_some())
            .map(|(provider, region)| (provider.unwrap(), region))
            .map(|(provider, region)| (provider, DcCity::from_region(region, provider)))
            .filter(|(_, city)| city.is_some())
            .map(|(provider, city)| (provider, city.unwrap()))
    }

    /// Returns the provider and city name of the datacenter.
    pub fn _city_name(&self) -> Result<(&'static str, &'static str), ()> {
        self.city()
            .map(|(p, c)| Ok((p.as_str(), c.as_str())))
            .unwrap_or(Err(()))
    }

    /// Returns the name of the continent in which the datacenter is located.
    pub fn continent_name(&self) -> Option<&'static str> {
        self.city().map(|(_, c)| match c {
            DcCity::Boardman | DcCity::Newark | DcCity::Seattle | DcCity::Washington => {
                "North America"
            }
            DcCity::Frankfurt | DcCity::London | DcCity::Nuremberg => "Europe",
            DcCity::SaoPaulo => "South America",
            DcCity::Mumbai | DcCity::Singapore | DcCity::Tokyo => "Asia",
            DcCity::Sydney => "Oceania",
        })
    }

    /// Converts AWS region into `CloudDatacenter`.
    #[cfg(feature = "aws")]
    pub fn from_aws_region(region: &str) -> Self {
        Self(format!("{}/{region}", DcProvider::Aws.as_str()))
    }

    /// Converts canonical string, such as "Linode/Newark", into `CloudDatacenter`.
    pub fn from_canonical(canonical_path: &str) -> Self {
        Self(
            canonical_path
                .split_once("/")
                .map(|(provider_name, city_name)| {
                    let provider: Option<DcProvider> = provider_name.parse().ok();
                    let city: Option<DcCity> = city_name.parse().ok();
                    (provider, city)
                })
                .filter(|(provider, city)| provider.is_some() && city.is_some())
                .map(|(provider, city)| (provider.unwrap(), city.unwrap()))
                .map(|(provider, city)| (provider.as_str(), city.to_region(provider)))
                .filter(|(_, region)| region.is_some())
                .map(|(provider_name, region)| (provider_name, region.unwrap()))
                .map(|(provider_name, region)| format!("{provider_name}/{region}"))
                .unwrap_or(canonical_path.to_owned()),
        )
    }

    /// Converts Hetzner region into `CloudDatacenter`.
    #[cfg(feature = "hetzner")]
    pub fn from_hetzner_region(region: &str) -> Self {
        Self(format!("{}/{region}", DcProvider::Hetzner.as_str()))
    }

    /// Converts Linode region into `CloudDatacenter`.
    #[cfg(feature = "linode")]
    pub fn from_linode_region(region: &str) -> Self {
        Self(format!("{}/{region}", DcProvider::Linode.as_str()))
    }

    /// Returns AWS region nearest to `CloudDatacenter` for geo IP.
    #[cfg(feature = "aws")]
    pub fn nearest_aws_region(&self) -> &'static str {
        self.city()
            .and_then(|(_, c)| match c {
                // Boardman, OR is south of Seattle.
                DcCity::Seattle => Some("us-west-2"),
                // Nuremberg is near Frankfurt.
                DcCity::Nuremberg => Some("eu-central-1"),
                _ => c.to_aws_region(),
            })
            .unwrap_or("us-east-1")
    }

    /// If `CloudDatacenter` is in AWS then return it otherwise error.
    pub fn to_aws_region(&self) -> Result<String, Error> {
        self.to_region(DcProvider::Aws)
    }

    /// Return `CloudDatacenter` in canonical format.  For example, "linode/Newark".
    pub fn to_canonical(&self) -> String {
        self.city()
            .map(|(provider, city)| (provider.as_display_name(), city.as_str()))
            .map(|(provider_name, city_name)| format!("{provider_name}/{city_name}"))
            .unwrap_or_else(|| self.0.clone())
    }

    /// If `CloudDatacenter` is in Hetzner then return it otherwise error.
    pub fn to_hetzner_region(&self) -> Result<String, Error> {
        self.to_region(DcProvider::Hetzner)
    }

    /// If `CloudDatacenter` is in Linode then return it otherwise error.
    pub fn to_linode_region(&self) -> Result<String, Error> {
        self.to_region(DcProvider::Linode)
    }

    fn to_region(&self, provider: DcProvider) -> Result<String, Error> {
        self.0
            .split_once("/")
            .filter(|(prefix, _)| *prefix == provider.as_str())
            .map(|(_, label)| label.to_owned())
            .ok_or(Error::Http(
                StatusCode::NOT_ACCEPTABLE,
                format!("{self}: not a datacenter in {provider}"),
            ))
    }
}

impl Default for CloudDatacenter {
    fn default() -> Self {
        Self("developer/computer".to_string())
    }
}

impl Display for CloudDatacenter {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(&self.0)
    }
}

impl FromStr for CloudDatacenter {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dc = Self(s.to_owned());
        if dc.city().is_none() {
            Err(Error::Http(
                StatusCode::NOT_FOUND,
                format!("{s}: not a supported datacenter"),
            ))
        } else {
            Ok(dc)
        }
    }
}
