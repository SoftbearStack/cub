// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

/// A wrapper around base 64 library.
mod b64;
/// Given a user agent `String` determine whether it is a web scaping bot.
mod bot;
/// A wrapper around Dynamo DB client updates.
mod ddbupdate;
/// A wrapper around Dynamo DB client.
mod dynamo;
/// A wrapper to run a router via AWS API Gateway and Lambda Proxy.
mod lambda;
/// A wrapper around S3 client.
mod s3;
/// Run an `axum::Router` on incoming requests from a socket.
mod socket;
/// Unit tests.
mod tests;
/// A wrapper around Translate client.
mod translate;
/// A wrapper to send messages to a websocket via AWS API Gateway.
mod websocket;

pub use crate::aws::b64::{b64_to_u64, u64_to_b64};
pub use crate::aws::bot::user_agent_is_bot;
pub use crate::aws::ddbupdate::{ddb_update, DynamoUpdateBuilder};
pub use crate::aws::dynamo::{
    create_aws_config_loader, create_ddb_item, delete_ddb_item, delete_ddb_ranged_item,
    describe_ddb_table_length, get_ddb_item, get_ddb_ranged_item, load_aws_config, new_ddb_client,
    put_ddb_item, query_ddb, query_ddb_hash_range, scan_ddb, to_dynamo_av, to_dynamo_den,
    to_dynamo_des, to_dynamo_item, to_dynamo_sen, to_dynamo_ses, update_ddb_item, DynamoDbClient,
};
pub use crate::aws::lambda::{is_lambda_env, run_router_on_lambda};
pub use crate::aws::s3::{
    get_s3_item, list_s3_bucket, new_s3_client, presigned_s3_download_url, presigned_s3_upload_url,
    put_s3_item, S3Client,
};
pub use crate::aws::socket::run_router_on_socket;
pub use crate::aws::translate::{
    braces_valid, new_translate_client, translate_text, TranslateClient,
};
pub use crate::aws::websocket::{new_ws_client, send_ws_message, WebsocketClient};
