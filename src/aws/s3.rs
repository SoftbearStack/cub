// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use super::load_aws_config;
use crate::common::{CubConfig, Error};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use axum::http::StatusCode;
use std::time::Duration;

/// A convenient alias for S3 client so consuming code doesn't need to add it to `Cargo.toml`
pub type S3Client = aws_sdk_s3::Client;

/// Retrieves an object from S3.
pub async fn get_s3_item(client: &S3Client, bucket: &str, key: &str) -> Result<Vec<u8>, Error> {
    let mut object = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| Error::Anyhow(e.into(), format!("get_s3_item({bucket}, {key})")))?;

    let mut buf: Vec<u8> = Vec::with_capacity(10 * 1024 * 1024);
    while let Some(bytes) = object
        .body
        .try_next()
        .await
        .map_err(|e| Error::Anyhow(e.into(), format!("s3_try_next({bucket}, {key}")))?
    {
        buf.extend_from_slice(&bytes);
    }

    Ok(buf.into())
}

/// Lists objects in the specified S3 bucket.
pub async fn list_s3_bucket(client: &Client, bucket: &str) -> Result<Vec<String>, Error> {
    let output = client
        .list_objects_v2()
        .bucket(bucket)
        .send()
        .await
        .map_err(|e| Error::Anyhow(e.into(), format!("list_s3_bucket({bucket}")))?;
    if output.is_truncated().unwrap_or(false) {
        Err(Error::Http(
            StatusCode::FAILED_DEPENDENCY,
            format!("{bucket}: S3 result truncated"),
        ))
    } else {
        Ok(output
            .contents()
            .into_iter()
            .map(|obj| obj.key().unwrap_or_default().into())
            .collect::<Vec<_>>())
    }
}

/// Creates an S3 client.
pub async fn new_s3_client(cub_config: &CubConfig) -> S3Client {
    let aws_config = load_aws_config(cub_config).await;
    Client::new(&aws_config)
}

/// Retrieves the pre-signed URL for an object from S3.
pub async fn presigned_s3_download_url(
    client: &S3Client,
    bucket: &str,
    key: &str,
) -> Result<String, Error> {
    // Expires in 15 minutes aka 900 seconds.
    let expiry = PresigningConfig::expires_in(Duration::from_secs(900))
        .map_err(|e| Error::Anyhow(e.into(), format!("presigning_config({bucket}, {key}")))?;
    let presigned_request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .presigned(expiry)
        .await
        .map_err(|e| {
            Error::Anyhow(
                e.into(),
                format!("presigned_s3_download_url({bucket}, {key}"),
            )
        })?;
    Ok(presigned_request.uri().to_string())
}

/// Retrieves the pre-signed URL for an object from S3.
pub async fn presigned_s3_upload_url(
    client: &S3Client,
    bucket: &str,
    key: &str,
) -> Result<String, Error> {
    // Expires in 15 minutes aka 900 seconds.
    let expiry = PresigningConfig::expires_in(Duration::from_secs(900))
        .map_err(|e| Error::Anyhow(e.into(), format!("presigning_config({bucket}, {key}")))?;
    let presigned_request = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .presigned(expiry)
        .await
        .map_err(|e| Error::Anyhow(e.into(), format!("presigned_s3_upload_url({bucket}, {key}")))?;
    Ok(presigned_request.uri().to_string())
}

/// Put an object into the specified S3 bucket.
pub async fn put_s3_item(
    client: &S3Client,
    bucket: &str,
    key: &str,
    data: Vec<u8>,
    content_type: &str,
) -> Result<(), Error> {
    client
        .put_object()
        .bucket(bucket)
        .content_type(content_type)
        .key(key)
        .body(ByteStream::from(data))
        .send()
        .await
        .map_err(|e| Error::Anyhow(e.into(), format!("put_s3_item({bucket}, {key}")))?;
    Ok(())
}
