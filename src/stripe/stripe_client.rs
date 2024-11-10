// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::common::{CubConfig, Error};
use core::fmt::Debug;
use hyper::header::{HeaderMap, HeaderValue};
use hyper::{Method, StatusCode};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEBUG_REQUEST: bool = false;
const DEBUG_RESPONSE: bool = false;

/// Stripe HTTP Client.
pub struct StripeClient {
    client: reqwest::Client,
}

impl StripeClient {
    /// Create Stripe HTTP Client.
    pub fn new(cub_config: &CubConfig) -> Self {
        #[derive(Deserialize)]
        struct StripeConfig {
            secret_key: String,
        }
        #[derive(Deserialize)]
        struct ConfigToml {
            stripe: StripeConfig,
        }
        let ConfigToml {
            stripe: StripeConfig { secret_key },
        } = cub_config.get().expect("stripe.toml");

        let mut default_headers = HeaderMap::new();
        let mut auth_header = HeaderValue::from_str(&format!("Bearer {}", secret_key)).unwrap();
        auth_header.set_sensitive(true);
        default_headers.insert(reqwest::header::AUTHORIZATION, auth_header);

        let client = Client::builder()
            .timeout(Duration::from_secs(20))
            .default_headers(default_headers)
            .build()
            .unwrap();
        Self { client }
    }

    /// Delete the object with the specified path from Stripe.
    pub(crate) async fn delete(&self, path: &str) -> Result<(), Error> {
        let request_path = format!("https://api.stripe.com/v1/{path}");
        if DEBUG_REQUEST {
            println!(">> DELETE {request_path}");
        }
        let request = self.client.request(Method::DELETE, request_path);
        match request.send().await {
            Ok(r) => {
                let status = r.status();
                if status.is_success() {
                    Ok(())
                } else {
                    match r.text().await {
                        Ok(body) => Err(Error::Http(status, format!("stripe delete: {body}"))),
                        Err(e) => Err(Error::Http(status, format!("stripe delete: {e}"))),
                    }
                }
            }
            Err(e) => Err(Error::Http(
                StatusCode::SERVICE_UNAVAILABLE,
                format!("stripe delete: {e}"),
            )),
        }
    }

    /// Get the object with the specified path from Stripe.
    pub(crate) async fn get<T: Debug + DeserializeOwned>(&self, path: &str) -> Result<T, Error> {
        let request_path = format!("https://api.stripe.com/v1/{path}");
        if DEBUG_REQUEST {
            println!(">> GET {request_path}");
        }
        let request = self.client.request(Method::GET, request_path);
        match request.send().await {
            Ok(r) => {
                let status = r.status();
                if status.is_success() {
                    match r.json().await {
                        Ok(response) => {
                            if DEBUG_RESPONSE {
                                println!("{response:?} (code {status})");
                            }
                            Ok(response)
                        }
                        Err(e) => Err(Error::Http(status, format!("stripe JSON: {e}"))),
                    }
                } else {
                    match r.text().await {
                        Ok(body) => Err(Error::Http(status, format!("stripe GET: {body}"))),
                        Err(e) => Err(Error::Http(status, format!("stripe GET: {e}"))),
                    }
                }
            }
            Err(e) => Err(Error::Http(
                StatusCode::SERVICE_UNAVAILABLE,
                format!("stripe GET: {e}"),
            )),
        }
    }

    /// Post URL encoded form to Stripe via Stripe client.
    pub(crate) async fn post<F: Debug + Serialize, T: Debug + DeserializeOwned>(
        &self,
        path: &str,
        payload: &F,
    ) -> Result<T, Error> {
        let request_path = format!("https://api.stripe.com/v1/{path}");
        if DEBUG_REQUEST {
            println!(">> POST {request_path}\n{:?}", payload);
        }
        let request = self
            .client
            .request(Method::POST, request_path)
            .form(payload);
        match request.send().await {
            Ok(r) => {
                let status = r.status();
                if status.is_success() {
                    match r.json().await {
                        Ok(response) => {
                            if DEBUG_RESPONSE {
                                println!("{response:?} (code {status})");
                            }
                            Ok(response)
                        }
                        Err(e) => Err(Error::Http(
                            StatusCode::NOT_ACCEPTABLE,
                            format!("stripe JSON: {e}"),
                        )),
                    }
                } else {
                    match r.text().await {
                        Ok(body) => Err(Error::Http(status, format!("stripe POST: {body}"))),
                        Err(e) => Err(Error::Http(status, format!("stripe POST: {e}"))),
                    }
                }
            }
            Err(e) => Err(Error::Http(
                StatusCode::SERVICE_UNAVAILABLE,
                format!("stripe POST: {e}"),
            )),
        }
    }
}

/// Create a Stripe Client.
pub fn new_stripe_client(cub_config: &CubConfig) -> StripeClient {
    StripeClient::new(cub_config)
}
