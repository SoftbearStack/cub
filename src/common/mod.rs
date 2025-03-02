// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

mod auth;
#[cfg(feature = "toml")]
mod config;
/// An enum that encapsulates a variety of error types.
mod error;
#[cfg(feature = "hyper")]
mod http;

#[cfg(feature = "aws")]
pub use self::auth::{AuthenticatedId, Identity, UserName};
#[cfg(feature = "toml")]
pub use self::config::CubConfig;
pub use self::error::Error;
#[cfg(feature = "aws")]
pub use self::error::{AnyhowError, DynamoError, SerdeError};
#[cfg(feature = "hyper")]
pub use self::http::create_error_response;
