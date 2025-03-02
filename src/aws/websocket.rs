// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use super::create_aws_config_loader;
use crate::common::{CubConfig, Error};
use aws_sdk_apigatewaymanagement::primitives::Blob;
use aws_sdk_apigatewaymanagement::Client;
use serde::Deserialize;

/// A convenient alias for websocket client so consuming code doesn't need to add it to `Cargo.toml`
pub type WebsocketClient = aws_sdk_apigatewaymanagement::Client;

/// Creates a websocket client.
pub async fn new_ws_client(cub_config: &CubConfig) -> WebsocketClient {
    #[derive(Deserialize)]
    struct AwsConfig {
        ws_endpoint_url: String,
    }
    #[derive(Deserialize)]
    struct ConfigToml {
        aws: AwsConfig,
    }
    let mut config_loader = create_aws_config_loader(cub_config);
    if let Ok(ConfigToml {
        aws: AwsConfig { ws_endpoint_url },
    }) = cub_config.get()
    {
        config_loader = config_loader.endpoint_url(ws_endpoint_url);
    };
    let aws_config = config_loader.load().await;
    Client::new(&aws_config)
}

/// Send a message to the specified websocket.
pub async fn send_ws_message(
    client: &WebsocketClient,
    connection_id: &str,
    message: &[u8],
) -> Result<(), Error> {
    client
        .post_to_connection()
        .connection_id(connection_id)
        .data(Blob::new(message))
        .send()
        .await
        .map_err(|e| Error::Anyhow(e.into(), format!("send_ws_message({connection_id})")))?;
    Ok(())
}
