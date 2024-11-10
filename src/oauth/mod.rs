// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

mod discord;
/// A wrapper around a set of OAuth2 provider APIs.
mod oauth;

pub use self::oauth::{new_oauth_client, OAuthClient, OAuthProvider, Url};
