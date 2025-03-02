// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
/// Customer address.
pub struct BillingAddress {
    /// City, e.g. "Portland".
    pub city: Option<String>,

    /// Two-letter ISO 3166-1 country code, e.g. "US".
    pub country: Option<String>,

    /// Address line 1 (Street address), e.g. "123 First Ave".
    pub line1: Option<String>,

    /// Address line 2 (Apartment, Suite or Unit), e.g. "Apt 123".
    pub line2: Option<String>,

    /// ZIP or postal code, e.g. "11201".
    pub postal_code: Option<String>,

    /// State, e.g. "WA".
    pub state: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
/// Customer billing details (address, email, name, and phone).
pub struct BillingDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The customer's billing address.
    pub address: Option<BillingAddress>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The customer's email address, e.g. "foo@gmail.com".
    pub email: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The customer's full name, e.g. "John Doe".
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The customer's phone number, e.g. "2125551212".
    pub phone: Option<String>,
}
