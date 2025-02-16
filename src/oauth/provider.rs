// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::Url;
use crate::common::{AuthenticatedId, Error, Identity};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::num::NonZeroU64;

/// The `OAuthProvider` enum contains the list of supported `OAuth2` providers.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash, Serialize)]
pub enum OAuthProvider {
    /// https://discord.com
    Discord,
    /// https://google.com
    Google,
}

impl Display for OAuthProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Discord => Display::fmt("Discord", f),
            Self::Google => Display::fmt("Google", f),
        }
    }
}

/// Convert from provider name to `OAuthProvider` enum.
impl TryFrom<String> for OAuthProvider {
    type Error = Error;
    fn try_from(oauth_provider: String) -> Result<Self, Error> {
        match oauth_provider.as_str() {
            "Discord" | "discord" => Ok(OAuthProvider::Discord),
            "Google" | "google" => Ok(OAuthProvider::Google),
            _ => Err(Error::String(format!(
                "{}: not an oauth2 provider",
                oauth_provider
            ))),
        }
    }
}

/// Cloud DNS trait
#[async_trait]
pub trait OAuthService {
    /// Handles the callback from an OAuth2 provider.
    async fn authenticated(&self, code: String) -> Result<Identity, Error>;
    /// For diagnostic purposes.
    async fn authenticated_by_localhost(&self, code: String) -> Result<Identity, Error>;
    /// Returns provider-specific details.
    async fn detail(&self, oauth_id: Option<&AuthenticatedId>, name: &str)
        -> Result<String, Error>;
    /// Returns provider.
    fn provider(&self) -> OAuthProvider;
    /// Returns a `Url` that redirects to the specified OAuth2 provider.
    fn redirect(&self) -> Url;
    /// For diagnostic purposes.  Only supported for Discord.
    fn redirect_to_localhost(&self) -> Url;
    /// Sends a message via the provider, if possible.
    async fn send_message(
        &self,
        channel_name: &str,
        message: &str,
        ping: bool,
        reply_to_id: Option<NonZeroU64>,
    ) -> Result<(), Error>;
}
