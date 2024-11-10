// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::{Display, Formatter};

#[cfg(feature = "aws")]
/// A convenient alias for Anyhow so consuming code doesn't need to add to `Cargo.toml`
pub type AnyhowError = anyhow::Error;

#[cfg(feature = "aws")]
/// A convenient alias for Dynamo DB error so consuming code doesn't need to add to `Cargo.toml`
pub type DynamoError = aws_sdk_dynamodb::Error;

#[cfg(feature = "aws")]
/// A convenient alias for Serde Dynamo error so consuming code doesn't need to add to `Cargo.toml`
pub type SerdeError = serde_dynamo::Error;

#[derive(Debug)]
/// An enum that encapsulates a variety of error types.
///
/// # Example
///
/// Error::Http(StatusCode::NOT_FOUND, format!("{path}: not found"))
pub enum Error {
    #[cfg(feature = "aws")]
    /// Anywow error
    Anyhow(AnyhowError, String),
    #[cfg(feature = "aws")]
    /// Dynamo (database) error
    Dynamo(DynamoError, String),
    /// HTTP (or miscellaneous) error
    #[cfg(feature = "hyper")]
    Http(hyper::StatusCode, String),
    #[cfg(feature = "aws")]
    /// Serde (serialization or deserialization) error
    Serde(SerdeError),
    /// String error.
    String(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            #[cfg(feature = "aws")]
            Error::Http(status_code, mesg) => Display::fmt(&format!("{status_code}: {mesg}"), f),
            #[cfg(feature = "aws")]
            Error::Dynamo(DynamoError::ConditionalCheckFailedException(_), source) => {
                Display::fmt(&format!("DynamoDb condition not met by {source}"), f)
            }
            Error::String(s) => Display::fmt(&s, f),
            #[cfg(feature = "aws")]
            _ => Display::fmt(&format!("{self:?}"), f),
        }
    }
}
