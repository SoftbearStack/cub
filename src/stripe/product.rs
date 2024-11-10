// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{PriceId, StripeClient};
use crate::common::Error;
use crate::impl_wrapper_str;
use crate::serde_utils::is_default;
use crate::time_id::NonZeroUnixSeconds;
use core::fmt::Debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Product ID.
pub struct ProductId(pub String);
impl_wrapper_str!(ProductId);

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Product.
pub struct Product {
    /// Unique identifier for the product.
    pub id: ProductId,

    #[serde(default, skip_serializing_if = "is_default")]
    /// Products may be active or archived.
    pub active: bool,

    /// Each SKU may have up to 5 attributes (e.g., `["color", "size"]`).
    #[serde(default, skip_serializing_if = "is_default")]
    pub attributes: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Date/Time record was created.
    pub created: Option<NonZeroUnixSeconds>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Default price.
    pub default_price: Option<PriceId>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// Products aren't actually deleted but are flagged as such.
    pub deleted: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Product description.
    pub description: Option<String>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// Up to 8 URLs of images of product.
    pub images: Vec<String>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// Live mode vs test mode.
    pub livemode: bool,

    #[serde(default, skip_serializing_if = "is_default")]
    /// Application specific metadata.
    pub metadata: HashMap<String, String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Product name, appears on subscription invoice.
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Description for CC statement.
    pub statement_descriptor: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Date/Time record was last updated.
    pub updated: Option<NonZeroUnixSeconds>,
}

impl StripeClient {
    /// List up to 100 products.
    pub async fn list_products(&self) -> Result<Vec<Product>, Error> {
        #[derive(Debug, Deserialize)]
        struct ProductList {
            data: Vec<Product>,
        }
        let mut list: ProductList = self.get("products?limit=100").await?;
        list.data.retain(|p| p.active || !p.deleted);
        Ok(list.data)
    }

    /// Load an existing Product.
    pub async fn load_product(&self, product_id: &ProductId) -> Result<Product, Error> {
        Ok(self.get(&format!("products/{product_id}")).await?)
    }
}
