// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::common::{AuthenticatedId, CubConfig, Identity};
use crate::oauth::discord;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;

/// A convenient alias for URL so consuming code doesn't need to add it to `Cargo.toml`
pub type Url = reqwest::Url;

/// The `OAuthProvider` enum contains the list of supported `OAuth2` providers.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum OAuthProvider {
    /// https://discord.com
    Discord,
}

/// Convert from provider name to `OAuthProvider` enum.
impl TryFrom<String> for OAuthProvider {
    type Error = String;
    fn try_from(oauth_provider: String) -> Result<Self, String> {
        match oauth_provider.as_str() {
            "discord" => Ok(OAuthProvider::Discord),
            // "google" => Ok(OAuthProvider::Google),
            _ => Err(format!("{}: not an oauth2 provider", oauth_provider)),
        }
    }
}

/// The `OAuthClient` calls the necessary OAuth2 provider APIs to authenticate a user.
pub struct OAuthClient {
    discord: discord::OAuth2Service,
    // Google: google::OAuth2Service,
}

impl OAuthClient {
    /// Returns a new Oauth2 wrapper service.
    pub fn new(cub_config: &CubConfig) -> Self {
        Self {
            discord: discord::OAuth2Service::new(cub_config),
            // google::OAuth2Service::new(vault),
        }
    }

    /// Handles the callback from an OAuth2 provider.
    pub async fn authenticated(
        &self,
        provider: OAuthProvider,
        code: String,
    ) -> Result<Identity, String> {
        match provider {
            OAuthProvider::Discord => self.discord.authenticated(code).await,
            // OAuthProvider::Google => self.google.authenticated(code).await,
        }
    }

    /// For diagnostic purposes.  Only supported for Discord.
    pub async fn authenticated_by_localhost(
        &self,
        provider: OAuthProvider,
        code: String,
    ) -> Result<Identity, String> {
        match provider {
            OAuthProvider::Discord => self.discord.authenticated_by_localhost(code).await,
            // _ => self.authenticated(provider, code),
        }
    }

    /// Returns provider-specific details.
    pub async fn detail(
        &self,
        provider: OAuthProvider,
        oauth_id: Option<&AuthenticatedId>,
        name: &str,
    ) -> Result<String, String> {
        match provider {
            OAuthProvider::Discord => self.discord.detail(oauth_id, name).await,
            // OAuthProvider::Google => self.google.detail(oauth_id, name).await,
        }
    }

    /// Returns a `Url` that redirects to the specified OAuth2 provider.
    pub fn redirect(&self, provider: OAuthProvider) -> Url {
        match provider {
            OAuthProvider::Discord => self.discord.redirect(),
            // OAuthProvider::Google => self.google.redirect(),
        }
    }

    /// For diagnostic purposes.  Only supported for Discord.
    pub fn redirect_to_localhost(&self, provider: OAuthProvider) -> Url {
        match provider {
            OAuthProvider::Discord => self.discord.redirect_to_localhost(),
            // _ => self.redirect(provider),
        }
    }

    /// Sends a message via the provider, if possible.
    pub async fn send_message(
        &self,
        provider: OAuthProvider,
        channel_name: &str,
        message: &str,
        ping: bool,
        reply_to_id: Option<NonZeroU64>,
    ) -> Result<(), String> {
        match provider {
            OAuthProvider::Discord => {
                self.discord
                    .send_message(channel_name, message, ping, reply_to_id)
                    .await
            } // OAuthProvider::Google => ...,
        }
    }
}

/// Creates an OAuth client.
pub fn new_oauth_client(cub_config: &CubConfig) -> OAuthClient {
    OAuthClient::new(cub_config)
}
