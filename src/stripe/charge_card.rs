// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use crate::time_id::NonZeroUnixSeconds;
use core::fmt::Debug;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Credit or debit card brand, e.g. `Visa`, `American Express`, etc.
pub enum Brand {
    #[serde(rename = "American Express")]
    /// AMEX card
    AmericanExpress,
    #[serde(rename = "Diners Club")]
    /// Diners card
    DinersClub,
    #[serde(rename = "Discover")]
    /// Discover card
    Discover,
    #[serde(rename = "JCB")]
    /// JCB
    JCB,
    #[serde(rename = "Visa")]
    /// Visa card
    Visa,
    #[serde(rename = "MasterCard")]
    /// Mastercard
    MasterCard,
    #[serde(rename = "UnionPay")]
    /// Union Pay
    UnionPay,
    #[serde(other)]
    #[serde(rename = "Unknown")]
    /// Other not yet supported brand.
    Unknown,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Credit Card or Debit Card
pub struct ChargeCard {
    /// Charge card brand, e.g. `Visa`, `American Express`, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand: Option<Brand>,

    /// The result of checks to validate address line1, postal code, or CVC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<Checks>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Two-letter ISO 3166-1 country code of billing address if provided, e.g. "US".
    pub country: Option<String>,

    /// Date/time record was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<NonZeroUnixSeconds>,

    /// Credit or debit card CVC (for upload to Stripe only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cvc: Option<u16>,

    /// Credit or debit card expiration month, e.g. 4.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp_month: Option<u8>,

    /// Four-digit credit or debit card expiration year, e.g. 2023.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp_year: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Fingerprint to check whether two CC numbers are identical without knowing the numbers.
    pub fingerprint: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Funding type may be `credit`, `debit`, `prepaid`, or `unknown`.
    pub funding: Option<Funding>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The last four digits of the credit or debit card number, e.g. "1234".
    pub last4: Option<String>,

    /// Credit or debit card card number (for upload to Stripe only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// If a credit or debit card check is performed, the result may be: `pass`, `fail`, `unavailable`, or `unchecked`.
pub enum CheckResult {
    #[serde(rename = "pass")]
    /// Check passed.
    Pass,
    #[serde(rename = "fail")]
    /// Check failed.
    Failed,
    #[serde(rename = "unavailable")]
    /// Check result unavailable.
    Unavailable,
    #[serde(rename = "unchecked")]
    /// Check was not performed.
    Unchecked,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
/// Credit or debit card validation checks.
pub struct Checks {
    /// If `address_line1` was provided, the check result may be: `pass`, `fail`, `unavailable`, or `unchecked`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line1_check: Option<CheckResult>,

    /// If `address_zip` was provided, the check result may be: `pass`, `fail`, `unavailable`, or `unchecked`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_postal_code_check: Option<CheckResult>,

    /// If `CVC` was provided, the check result may be: `pass`, `fail`, `unavailable`, or `unchecked`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cvc_check: Option<CheckResult>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Funding type, e.g. `credit`, `debit`, `prepaid`, or `unknown`.
pub enum Funding {
    #[serde(rename = "credit")]
    /// Credit card.
    Credit,
    #[serde(rename = "debit")]
    /// Debit card.
    Debit,
    #[serde(rename = "prepaid")]
    /// Prepaid card.
    Prepaid,
    #[serde(other)]
    #[serde(rename = "unknown")]
    /// Other not yet supported card type.
    Unknown,
}
