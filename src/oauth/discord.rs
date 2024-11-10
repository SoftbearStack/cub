// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::Url;
use crate::common::{AuthenticatedId, CubConfig, Identity, UserName};
use crate::serde_utils::is_default;
use hyper::header::{HeaderMap, HeaderValue};
use oauth2::basic::{BasicClient, BasicTokenType};
use oauth2::{
    reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    EmptyExtraTokenFields, RedirectUrl, Scope, StandardTokenResponse, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroU64;
use std::time::Duration;

const DEBUG: bool = false;

pub struct OAuth2Service {
    guild_id: NonZeroU64,
    http_auth_client: reqwest::Client,
    http_api_client: reqwest::Client,
    localhost_redirect_url: Option<String>,
    oauth2_client: BasicClient,
}

impl OAuth2Service {
    pub fn new(cub_config: &CubConfig) -> Self {
        #[derive(Deserialize)]
        struct DiscordConfig {
            bot_token: String,
            client_id: String,
            client_secret: String,
            guild_id: String,
            localhost_redirect_url: Option<String>,
            redirect_url: String,
        }
        #[derive(Deserialize)]
        struct ConfigToml {
            discord: DiscordConfig,
        }
        let ConfigToml {
            discord:
                DiscordConfig {
                    bot_token,
                    client_id,
                    client_secret,
                    guild_id,
                    localhost_redirect_url,
                    redirect_url,
                },
        } = cub_config.get().expect("discord.toml");

        let bot_token_header = HeaderValue::from_str(&format!("Bot {}", bot_token))
            .map(|h| {
                let mut default_headers = HeaderMap::new();
                default_headers.insert(reqwest::header::AUTHORIZATION, h);
                default_headers
            })
            .expect("invalid Discord bot token");

        let auth_url = String::from("https://discord.com/api/oauth2/authorize?response_type=code");
        let token_url = String::from("https://discord.com/api/oauth2/token");

        let guild_id = NonZeroU64::new(guild_id.parse::<u64>().expect("invalid Discord guild ID"))
            .expect("Discord guild ID was 0");
        let http_api_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .default_headers(bot_token_header)
            .build()
            .unwrap();
        let http_auth_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(4))
            .build()
            .unwrap();
        let oauth2_client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new(auth_url).unwrap(),
            Some(TokenUrl::new(token_url).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_url).expect("invalid redirect URL"));

        Self {
            guild_id,
            http_api_client,
            http_auth_client,
            localhost_redirect_url,
            oauth2_client,
        }
    }

    pub async fn authenticated(&self, code: String) -> Result<Identity, String> {
        self.auth_token_to_identity(
            self.oauth2_client
                .exchange_code(AuthorizationCode::new(code))
                .request_async(async_http_client)
                .await
                .map_err(|e| e.to_string())?,
        )
        .await
    }

    // For diagnostic purposes.
    pub async fn authenticated_by_localhost(&self, code: String) -> Result<Identity, String> {
        let Some(localhost_redirect_url) = self.localhost_redirect_url.clone() else {
            return self.authenticated(code).await;
        };
        let Ok(url) = RedirectUrl::new(localhost_redirect_url) else {
            return self.authenticated(code).await;
        };
        let client = self.oauth2_client.clone().set_redirect_uri(url);
        self.auth_token_to_identity(
            client
                .exchange_code(AuthorizationCode::new(code))
                .request_async(async_http_client)
                .await
                .map_err(|e| e.to_string())?,
        )
        .await
    }

    pub async fn detail(
        &self,
        oauth_id: Option<&AuthenticatedId>,
        name: &str,
    ) -> Result<String, String> {
        match (oauth_id, name) {
            (Some(oauth_id), "roles") => {
                let discord_id = Self::parse_oauth_id(oauth_id)?;
                Ok(self
                    .get_roles_csv(discord_id)
                    .await
                    .map_err(|e| format!("cannot get Discord roles: {e}"))?)
            }
            _ => Err(format!("{name}: not a supported detail for Discord")),
        }
    }

    async fn auth_token_to_identity(
        &self,
        token: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    ) -> Result<Identity, String> {
        if DEBUG {
            println!(
                "discord token expiry: {:?}",
                token.expires_in().map(|d| d.as_secs() / 3600)
            );
            println!(
                "discord refresh token: {:?}",
                token.refresh_token().map(|r| r.secret())
            );
        }

        // https://discord.com/developers/docs/resources/user#user-object-user-structure
        #[derive(Debug, Deserialize)]
        struct User {
            id: String,
            username: String,
            discriminator: String,
        }

        let user: User = self
            .http_auth_client
            .get("https://discord.com/api/users/@me")
            .timeout(Duration::from_secs(5))
            .bearer_auth(token.access_token().secret())
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<User>()
            .await
            .map_err(|e| e.to_string())?;

        let parsed = user.id.parse::<u64>().map_err(|e| e.to_string())?;
        let discord_id = NonZeroU64::new(parsed).ok_or_else(|| String::from("discord id was 0"))?;

        Ok(Identity {
            login_id: AuthenticatedId(format!("discord/{}", discord_id)),
            user_name: Some(UserName(if user.discriminator == "0" {
                user.username
            } else {
                format!("{}#{}", user.username, user.discriminator)
            })),
        })
    }

    async fn get_roles_csv(&self, discord_id: NonZeroU64) -> Result<String, String> {
        // https://discord.com/developers/docs/resources/guild#guild-member-object
        #[derive(Debug, Deserialize)]
        struct Membership {
            roles: Vec<String>,
        }

        let members_endpoint = format!(
            "https://discord.com/api/guilds/{}/members/{}",
            self.guild_id, discord_id
        );
        if DEBUG {
            // println!("members_endpoint is {}", members_endpoint);
        }

        let response = self
            .http_api_client
            .get(members_endpoint)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status_code = response.status();
        if status_code != reqwest::StatusCode::OK {
            let text = response.text().await.map_err(|e| e.to_string())?;
            let error = format!("Discord members error {status_code}: {text}");
            return Err(error);
        }
        let membership: Membership = response
            .json::<Membership>()
            .await
            .map_err(|e| e.to_string())?;

        if DEBUG {
            println!("membership is {:?}", membership);
        }

        #[derive(Debug, Deserialize)]
        struct Role {
            id: String,
            name: String,
        }

        let roles_endpoint = format!("https://discord.com/api/guilds/{}/roles", self.guild_id);
        if DEBUG {
            println!("roles_endpoint is {}", roles_endpoint);
        }

        let roles: Vec<Role> = self
            .http_api_client
            .get(roles_endpoint)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Vec<Role>>()
            .await
            .map_err(|e| e.to_string())?;

        if DEBUG {
            println!("roles are {:?}", roles);
        }

        let roles_hash: HashMap<String, String> =
            roles.into_iter().map(|role| (role.id, role.name)).collect();

        let user_roles: Vec<_> = membership
            .roles
            .iter()
            .map(|id| roles_hash.get(&*id))
            .filter(|name| name.is_some())
            .map(|name| name.unwrap().to_owned())
            .collect();
        let roles_csv = user_roles.join(",");
        if DEBUG {
            println!("roles are {roles_csv}");
        }

        Ok(roles_csv)
    }

    fn parse_oauth_id(oauth_id: &AuthenticatedId) -> Result<NonZeroU64, String> {
        let Some((prefix, discord_id_s)) = oauth_id.as_str().split_once('/') else {
            return Err(format!("{oauth_id}: invalid oauth ID"));
        };
        if prefix != "discord" {
            return Err(format!("{oauth_id}: not a Discord ID"));
        }
        discord_id_s
            .parse()
            .map_err(|_| format!("{oauth_id}: invalid number"))
    }

    pub fn redirect(&self) -> Url {
        let (auth_url, _csrf_token) = self
            .oauth2_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("identify".to_string()))
            .url();
        auth_url
    }

    // For diagnostic purposes.
    pub fn redirect_to_localhost(&self) -> Url {
        let Some(localhost_redirect_url) = self.localhost_redirect_url.clone() else {
            return self.redirect();
        };
        let Ok(url) = RedirectUrl::new(localhost_redirect_url) else {
            return self.redirect();
        };
        let client = self.oauth2_client.clone().set_redirect_uri(url);
        let (auth_url, _csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("identify".to_string()))
            .url();
        auth_url
    }

    pub async fn send_message(
        &self,
        channel_name: &str,
        message: &str,
        ping: bool,
        reply_to_id: Option<NonZeroU64>,
    ) -> Result<(), String> {
        #[derive(Deserialize)]
        struct Channel {
            id: String,
            name: String,
        }

        let channels: Vec<Channel> = self
            .http_api_client
            .get(format!(
                "https://discord.com/api/guilds/{}/channels",
                self.guild_id
            ))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Vec<Channel>>()
            .await
            .map_err(|e| e.to_string())?;

        let channel_id = channels
            .into_iter()
            .find(|c| c.name == channel_name)
            .map(|c| c.id)
            .ok_or_else(|| String::from("could not find channel"))?;

        #[derive(Serialize)]
        struct MessageReference {
            message_id: String,
        }

        const SUPPRESS_NOTIFICATIONS: u64 = 4096;

        #[derive(Serialize)]
        struct CreateMessage<'a> {
            content: &'a str,
            message_reference: Option<MessageReference>,
            #[serde(skip_serializing_if = "is_default")]
            flags: u64,
        }

        let create_message = CreateMessage {
            content: message,
            message_reference: reply_to_id.map(|id| MessageReference {
                message_id: id.to_string(),
            }),
            flags: if ping { 0 } else { SUPPRESS_NOTIFICATIONS },
        };

        self.http_api_client
            .post(format!(
                "https://discord.com/api/channels/{}/messages",
                channel_id
            ))
            .json(&create_message)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        self.http_api_client
            .post(format!(
                "https://discord.com/api/channels/{}/messages",
                channel_id
            ))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
