// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

/// A client of one or more OAuth2 provider APIs.
mod client;
mod discord;
mod google;
/// A wrapper around a particular OAuth2 provider API.
mod provider;

pub use self::client::{new_oauth_client, OAuthClient, Url};
pub use self::provider::{OAuthProvider, OAuthService};
