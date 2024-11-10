// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{ProductId, StripeClient};
use crate::common::Error;
use crate::impl_wrapper_str;
use crate::serde_utils::is_default;
use crate::time_id::NonZeroUnixSeconds;
use core::fmt::Debug;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Price ID.
pub struct PriceId(pub String);
impl_wrapper_str!(PriceId);

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// 3-letter currency designation e.g. Currency::USD.
pub enum Currency {
    #[serde(rename = "cad")]
    /// Canadian Dollar
    CAD,
    #[serde(rename = "eur")]
    /// Euro
    EUR,
    #[serde(rename = "gbp")]
    /// Great Britain Pound
    GBP,
    #[serde(rename = "usd")]
    /// United States Dollar
    USD,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
/// Whether the payment is one time or recurring.
pub enum PriceType {
    /// One time payment
    OneTime,
    /// Recurring (subscription) payment
    Recurring,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Price.
pub struct Price {
    /// Unique identifier for the price.
    pub id: PriceId,

    #[serde(default, skip_serializing_if = "is_default")]
    /// Prices may be active or archived.
    pub active: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Date/Time record was created.
    pub created: Option<NonZeroUnixSeconds>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// 3 letter IS-4217 currency code, e.g. Currency::USD.
    pub currency: Option<Currency>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// Prices aren't actually deleted but are flagged as such.
    pub deleted: bool,

    #[serde(default, skip_serializing_if = "is_default")]
    /// Live mode vs test mode.
    pub livemode: bool,

    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// e.g. PriceType::Recurring
    pub price_type: Option<PriceType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The product to which this price applies.
    pub product: Option<ProductId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Price expressed in cents.
    pub unit_amount: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Date/Time record was last updated.
    pub updated: Option<NonZeroUnixSeconds>,
}

impl StripeClient {
    /// List up to 100 prices.
    pub async fn list_prices(&self) -> Result<Vec<Price>, Error> {
        #[derive(Debug, Deserialize)]
        struct PriceList {
            data: Vec<Price>,
        }
        let mut list: PriceList = self.get("prices?limit=100").await?;
        list.data.retain(|p| p.active || !p.deleted);
        Ok(list.data)
    }

    /// Load an existing Price.
    pub async fn load_price(&self, price_id: &PriceId) -> Result<Price, Error> {
        Ok(self.get(&format!("prices/{price_id}")).await?)
    }
}
