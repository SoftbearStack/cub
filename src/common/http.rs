// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::Error;
use axum::body::Body;
use axum::response::{IntoResponse, Response};
use hyper::StatusCode;

/// Create a `Response` suitable for `axum::Router`.
pub fn create_error_response(status: StatusCode, message: String) -> Response {
    Response::builder()
        .status(status)
        .header("content-type", "text/plain")
        .body(Body::from(message))
        .unwrap()
        .into_response()
}

/// Convert `Error` into a `Response` suitable for `axum::Router`.
impl Into<Response> for Error {
    fn into(self) -> Response {
        match self {
            #[cfg(feature = "aws")]
            Error::Anyhow(e, s) => {
                create_error_response(StatusCode::FAILED_DEPENDENCY, format!("{s}: {e:?}"))
            }
            #[cfg(feature = "aws")]
            Error::Dynamo(e, s) => {
                create_error_response(StatusCode::FAILED_DEPENDENCY, format!("{s}: {e:?}"))
            }
            Error::Http(code, mesg) => create_error_response(code, mesg),
            Error::Serde(e) => {
                create_error_response(StatusCode::UNPROCESSABLE_ENTITY, format!("{e:?}"))
            }
            Error::String(s) => create_error_response(StatusCode::NOT_ACCEPTABLE, s),
        }
    }
}

impl Error {
    /// Map `String` to `Error`.
    pub fn from_string(s: String) -> Self {
        Error::Http(StatusCode::INTERNAL_SERVER_ERROR, s)
    }
}
