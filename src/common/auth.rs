// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
/// This is used, for example, with Oauth2 and JWT authentication.
pub struct AuthenticatedId(pub String);
crate::impl_wrapper_str!(AuthenticatedId);

/// The `Identity` struct is returned upon successful `OAuth2` authentication.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Identity {
    /// The login ID of the authenticated user.
    pub login_id: AuthenticatedId,
    /// The user name, if any, of the authenticated user.
    pub user_name: Option<UserName>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
/// A user name.
pub struct UserName(pub String);
crate::impl_wrapper_str!(UserName);
