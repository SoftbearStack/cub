// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{discord, google, OAuthProvider, OAuthService};
use crate::common::{AuthenticatedId, CubConfig, Error, Identity};
use std::collections::HashMap;
use std::num::NonZeroU64;

/// A convenient alias for URL so consuming code doesn't need to add it to `Cargo.toml`
pub type Url = reqwest::Url;

/// The `OAuthClient` calls the necessary OAuth2 provider APIs to authenticate a user.
pub struct OAuthClient {
    provider_clients: HashMap<OAuthProvider, Box<dyn OAuthService + Send + Sync>>,
}

impl OAuthClient {
    /// Returns a new Oauth2 wrapper service.
    pub fn new(cub_config: &CubConfig) -> Self {
        let mut provider_clients: HashMap<_, Box<dyn OAuthService + Send + Sync>> = HashMap::new();
        if let Ok(p) = discord::DiscordOAuth2Service::new(cub_config) {
            provider_clients.insert(p.provider(), Box::new(p));
        }
        if let Ok(p) = google::GoogleOAuth2Service::new(cub_config) {
            provider_clients.insert(p.provider(), Box::new(p));
        }
        Self { provider_clients }
    }

    /// Handles the callback from an OAuth2 provider.
    pub async fn authenticated(
        &self,
        provider: OAuthProvider,
        code: String,
    ) -> Result<Identity, Error> {
        self.get_provider_client(provider)?
            .authenticated(code)
            .await
    }

    /// For diagnostic purposes.
    pub async fn authenticated_by_localhost(
        &self,
        provider: OAuthProvider,
        code: String,
    ) -> Result<Identity, Error> {
        self.get_provider_client(provider)?
            .authenticated_by_localhost(code)
            .await
    }

    /// Returns provider-specific details.
    pub async fn detail(
        &self,
        provider: OAuthProvider,
        oauth_id: Option<&AuthenticatedId>,
        name: &str,
    ) -> Result<String, Error> {
        self.get_provider_client(provider)?
            .detail(oauth_id, name)
            .await
    }

    fn get_provider_client(
        &self,
        provider: OAuthProvider,
    ) -> Result<&(dyn OAuthService + Send + Sync), Error> {
        self.provider_clients
            .get(&provider)
            .map(|p| p.as_ref())
            .ok_or(Error::String(format!("{provider}: invalid provider")))
    }

    /// Enumerate supported OAuth providers.
    pub fn providers(&self) -> Vec<OAuthProvider> {
        self.provider_clients
            .iter()
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Returns a `Url` that redirects to the specified OAuth2 provider.
    pub fn redirect(&self, provider: OAuthProvider) -> Result<Url, Error> {
        Ok(self.get_provider_client(provider)?.redirect())
    }

    /// For diagnostic purposes.  Only supported for Discord.
    pub fn redirect_to_localhost(&self, provider: OAuthProvider) -> Result<Url, Error> {
        Ok(self.get_provider_client(provider)?.redirect_to_localhost())
    }

    /// Sends a message via the provider, if possible.
    pub async fn send_message(
        &self,
        provider: OAuthProvider,
        channel_name: &str,
        message: &str,
        ping: bool,
        reply_to_id: Option<NonZeroU64>,
    ) -> Result<(), Error> {
        self.get_provider_client(provider)?
            .send_message(channel_name, message, ping, reply_to_id)
            .await
    }
}

/// Creates an OAuth client.
pub fn new_oauth_client(cub_config: &CubConfig) -> OAuthClient {
    OAuthClient::new(cub_config)
}
