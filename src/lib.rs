// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

#![warn(missing_docs)]
//! This crate is a collection of useful code snippets that wrap and simplify
//! access to other crates.

#[cfg(feature = "aws")]
/// A wrapper which provides access to AWS Dynamo DB and AWS Lambda.
pub mod aws;
#[cfg(feature = "aws")]
pub use aws::*;

/// Types common to multiple wrappers.
pub mod common;
pub use common::*;

#[cfg(all(
    any(feature = "dns", feature = "hosts"),
    any(feature = "aws", feature = "hetzner", feature = "linode")
))]
/// Regions used with DNS and virtual hosting services.
pub mod datacenter;
#[cfg(all(
    any(feature = "dns", feature = "hosts"),
    any(feature = "aws", feature = "hetzner", feature = "linode")
))]
pub use datacenter::*;

#[cfg(feature = "dns")]
/// A wrapper which provides access to DNS services.
pub mod dns;
#[cfg(feature = "dns")]
pub use dns::*;

#[cfg(feature = "hosts")]
/// A wrapper which provides access to virtual hosting services.
pub mod hosts;
#[cfg(feature = "hosts")]
pub use hosts::*;

#[cfg(feature = "jwt")]
/// A wrapper which provides access to JWT validation and unpacking.
pub mod jwt;
#[cfg(feature = "jwt")]
pub use jwt::*;

#[cfg(feature = "log")]
/// Thread-safe logging.
pub mod log;
#[cfg(feature = "log")]
pub use log::*;

#[cfg(feature = "videos")]
/// A wrapper which provides access to virtual video services.
pub mod videos;
#[cfg(feature = "videos")]
pub use videos::*;

#[cfg(feature = "yew_markdown")]
/// Generate Yew from Markdown.
pub mod yew_markdown;
#[cfg(feature = "yew_markdown")]
pub use yew_markdown::*;

/// Macros used with `serde` serialization and serialization.
pub mod serde_utils;
pub use serde_utils::*;

#[cfg(feature = "stripe")]
/// A wrapper which provides access to Stripe payments.
pub mod stripe;
#[cfg(feature = "stripe")]
pub use stripe::*;

#[cfg(feature = "time_id")]
/// Generates Unix timestamps (in seconds or milliseconds) and also 16-, 32-, and 64-bit IDs.
pub mod time_id;
#[cfg(feature = "time_id")]
pub use time_id::*;

#[cfg(feature = "oauth")]
/// A wrapper which provides access to Oauth2 authentication.
pub mod oauth;
#[cfg(feature = "oauth")]
pub use oauth::*;
