// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{OAuthProvider, OAuthService, Url};
use crate::common::{AuthenticatedId, CubConfig, Error, Identity, UserName};
use async_trait::async_trait;
use reqwest::Method;
use serde::Deserialize;
use std::num::NonZeroU64;
use std::time::Duration;

pub struct GoogleOAuth2Service {
    client_id: String,
    client_secret: String,
    localhost_redirect_url: Option<String>,
    redirect_url: String,
}

impl GoogleOAuth2Service {
    pub fn new(cub_config: &CubConfig) -> Result<Self, Error> {
        #[derive(Deserialize)]
        struct GoogleConfig {
            client_id: String,
            client_secret: String,
            localhost_redirect_url: Option<String>,
            redirect_url: String,
        }
        #[derive(Deserialize)]
        struct ConfigToml {
            google: GoogleConfig,
        }
        let ConfigToml {
            google:
                GoogleConfig {
                    client_id,
                    client_secret,
                    localhost_redirect_url,
                    redirect_url,
                },
        } = cub_config.get().map_err(|e| Error::String(e.to_string()))?;
        Ok(Self {
            client_id,
            client_secret,
            localhost_redirect_url,
            redirect_url,
        })
    }

    async fn authenticated_by(
        &self,
        redirect_url: &String,
        code: &String,
    ) -> Result<Identity, Error> {
        let GoogleOAuth2Service {
            client_id,
            client_secret,
            ..
        } = self;

        let grant_type = "authorization_code".to_string();
        let token_payload: Vec<(&'static str, &String)> = vec![
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code", code),
            ("grant_type", &grant_type),
            ("redirect_uri", redirect_url),
        ];

        let http_client = Self::create_http_client()?;
        let token_endpoint = "https://oauth2.googleapis.com/token";
        let token_response = http_client
            .request(Method::POST, token_endpoint)
            .form(&token_payload)
            .send()
            .await
            .map_err(|e| Error::String(e.to_string()))?;
        if !token_response.status().is_success() {
            return match token_response.text().await {
                Ok(body) => Err(Error::String(format!("google token post: {body}"))),
                Err(e) => Err(Error::String(format!("token: {e}"))),
            };
        }
        #[derive(Deserialize)]
        struct GoogleTokenResponse {
            access_token: String,
        }
        let token_text = token_response
            .text()
            .await
            .map_err(|e| Error::String(format!("google token response: {e}")))?;
        let GoogleTokenResponse { access_token } = serde_json::from_str(&token_text)
            .map_err(|e| Error::String(format!("google token parse: {e}\n{token_text}")))?;

        let userinfo_endpoint = "https://www.googleapis.com/oauth2/v1/userinfo?alt=json";
        let userinfo_response = http_client
            .get(userinfo_endpoint)
            .header("Authorization", &format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| Error::String(e.to_string()))?;
        if !userinfo_response.status().is_success() {
            return match userinfo_response.text().await {
                Ok(body) => Err(Error::String(format!("userinfo: {body}"))),
                Err(e) => Err(Error::String(format!("userinfo: {e}"))),
            };
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct GoogleUserinfoResponse {
            #[serde(skip_serializing_if = "Option::is_none")]
            email: Option<String>,
            id: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<String>,
        }
        let userinfo_text = userinfo_response
            .text()
            .await
            .map_err(|e| Error::String(format!("userinfo response: {e}")))?;
        let GoogleUserinfoResponse { email, id, name } = serde_json::from_str(&userinfo_text)
            .map_err(|e| Error::String(format!("google userinfo parse: {e}\n{userinfo_text}")))?;
        let user_name = name.or(email);
        Ok(Identity {
            login_id: AuthenticatedId(format!("google/{}", id)),
            user_name: user_name.map(|u| UserName(u)),
        })
    }

    fn create_http_client() -> Result<reqwest::Client, Error> {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|e| Error::String(format!("cannot create http client: {e}")))
    }

    fn redirect_to(&self, redirect_url: &String) -> Url {
        let GoogleOAuth2Service { client_id, .. } = self;
        let response_type = "code";
        let scope = "openid email";
        let state = "1234"; // Not used.
        let v2_url = "accounts.google.com/o/oauth2/v2/auth";
        let auth_url = format!("https://{v2_url}?client_id={client_id}&redirect_uri={redirect_url}&response_type={response_type}&scope={scope}&state={state}");
        Url::parse(&auth_url).unwrap()
    }
}

#[async_trait]
impl OAuthService for GoogleOAuth2Service {
    async fn authenticated(&self, code: String) -> Result<Identity, Error> {
        let GoogleOAuth2Service { redirect_url, .. } = self;
        self.authenticated_by(redirect_url, &code).await
    }

    // For diagnostic purposes.
    async fn authenticated_by_localhost(&self, code: String) -> Result<Identity, Error> {
        let GoogleOAuth2Service {
            localhost_redirect_url,
            redirect_url,
            ..
        } = self;
        if let Some(localhost_redirect_url) = localhost_redirect_url {
            self.authenticated_by(localhost_redirect_url, &code).await
        } else {
            self.authenticated_by(redirect_url, &code).await
        }
    }

    async fn detail(
        &self,
        _oauth_id: Option<&AuthenticatedId>,
        name: &str,
    ) -> Result<String, Error> {
        Err(Error::String(format!(
            "{name}: not a supported detail for Google"
        )))
    }

    fn provider(&self) -> OAuthProvider {
        OAuthProvider::Google
    }

    fn redirect(&self) -> Url {
        let GoogleOAuth2Service { redirect_url, .. } = self;
        self.redirect_to(&redirect_url)
    }

    // For diagnostic purposes.
    fn redirect_to_localhost(&self) -> Url {
        let GoogleOAuth2Service {
            localhost_redirect_url,
            redirect_url,
            ..
        } = self;
        if let Some(localhost_redirect_url) = localhost_redirect_url {
            self.redirect_to(localhost_redirect_url)
        } else {
            self.redirect_to(redirect_url)
        }
    }

    async fn send_message(
        &self,
        channel_name: &str,
        _message: &str,
        _ping: bool,
        _reply_to_id: Option<NonZeroU64>,
    ) -> Result<(), Error> {
        Err(Error::String(format!(
            "{channel_name}: not a supported channel for Google"
        )))
    }
}
