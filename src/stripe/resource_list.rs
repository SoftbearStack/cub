// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Stripe resource list.
pub struct StripeResourceList<T> {
    /// List content.
    pub data: Vec<T>,
    /// Whether there is more content beyond what is in list.
    pub has_more: bool,
    /// Total count.
    pub total_count: Option<u64>,
    /// URL.
    pub url: String,
}

impl<T> Default for StripeResourceList<T> {
    fn default() -> Self {
        StripeResourceList {
            data: Vec::new(),
            has_more: false,
            total_count: None,
            url: String::new(),
        }
    }
}

impl<T: Clone> Clone for StripeResourceList<T> {
    fn clone(&self) -> Self {
        StripeResourceList {
            data: self.data.clone(),
            has_more: self.has_more,
            total_count: self.total_count,
            url: self.url.clone(),
        }
    }
}
