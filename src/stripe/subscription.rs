// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use super::{
    Currency, CustomerId, PaymentMethodId, Price, PriceId, StripeClient, StripeResourceList,
};
use crate::common::Error;
use crate::impl_wrapper_str;
use crate::serde_utils::is_default;
use crate::time_id::NonZeroUnixSeconds;
use core::fmt::Debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Subscription ID.
pub struct SubscriptionId(pub String);
impl_wrapper_str!(SubscriptionId);

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Subscription item ID.
pub struct SubscriptionItemId(pub String);
impl_wrapper_str!(SubscriptionItemId);

#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
/// Automatic tax.
pub struct AutomaticTax {
    #[serde(default, skip_serializing_if = "is_default")]
    /// Whether stripe automatically computes sales tax.
    pub enabled: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
/// Cancellation details.
pub struct CancellationDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Comment.
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Feedback.
    pub feedback: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Reason.
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
/// Collection method is typically `charge_automatically` but it can also be `send_invoice`.
pub enum CollectionMethod {
    /// Charge automatically.
    ChargeAutomatically,
    /// Send invoice.
    SendInvoice,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Subscription.
pub struct Subscription {
    /// Unique identifier for the subscription.
    pub id: SubscriptionId,

    #[serde(default, skip_serializing_if = "is_default")]
    /// Subscriptions may be active or archived.
    pub active: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Whether stripe automatically computes sales tax.
    pub automatic_tax: Option<AutomaticTax>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Date/Time of the first full invoice, and day of the month for subsequent invoices.
    pub billing_cycle_anchor: Option<NonZeroUnixSeconds>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Date/time subscription was canceled, if any.
    pub canceled_at: Option<NonZeroUnixSeconds>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Reason the subscription was canceled, if any.
    pub cancellation_details: Option<CancellationDetails>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Typically `charge_automatically` but it can also be `send_invoice`.
    pub collection_method: Option<CollectionMethod>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Date/Time record was created.
    pub created: Option<NonZeroUnixSeconds>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// 3 letter IS-4217 currency code, e.g. Currency::USD.
    pub currency: Option<Currency>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// End of current subscription period.
    pub current_period_end: Option<NonZeroUnixSeconds>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Start of current subscription period.
    pub current_period_start: Option<NonZeroUnixSeconds>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Customer paying for this subscription.
    pub customer: Option<CustomerId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Default payment method.
    pub default_payment_method: Option<PaymentMethodId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Subscription description.
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Date/time subscription ended, if any.
    pub ended_at: Option<NonZeroUnixSeconds>,

    /// List of subscription items, each with an attached plan.
    #[serde(default, skip_serializing_if = "is_default")]
    pub items: StripeResourceList<SubscriptionItem>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// Live mode vs test mode.
    pub livemode: bool,

    #[serde(default)]
    /// Application specific metadata.
    pub metadata: HashMap<String, String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Date the subscription started.
    pub start_date: Option<NonZeroUnixSeconds>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Subscription status.
    pub status: Option<SubscriptionStatus>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Subscription item.
pub struct SubscriptionItem {
    /// Unique identifier for the object.
    pub id: SubscriptionItemId,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Date/Time record was created.
    pub created: Option<NonZeroUnixSeconds>,

    #[serde(default)]
    /// Subscription items aren't actually deleted but are flagged as such.
    pub deleted: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Price
    pub price: Option<Price>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Quantity
    pub quantity: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
/// Subscription status, e.g. `active` or `canceled`.
pub enum SubscriptionStatus {
    /// Active.
    Active,
    /// Canceled.
    Canceled,
    /// Past due.
    PastDue,
    /// Incomplete.
    Incomplete,
    /// Incomplete expired.
    IncompleteExpired,
    /// Trailing.
    Trialing,
    /// Unpaid.
    Unpaid,
}

impl StripeClient {
    /// Create subscription for the specified customer and payment method.
    /// The price_id is linked to the subscription product.
    pub async fn create_subscription(
        &self,
        customer_id: &CustomerId,
        price_id: &PriceId,
        trial_period_days: u8,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<Subscription, Error> {
        // Assume collection_method is set to "charge_automatically" by default.
        // Assume customer has a default_payment_method, so it is not necessary to set one here.
        let mut form_data: Vec<(String, String)> = vec![
            ("customer".to_string(), customer_id.to_string()),
            ("items[0][price]".to_string(), price_id.to_string()),
            (
                "trial_period_days".to_string(),
                trial_period_days.to_string(),
            ),
        ];
        if let Some(metadata) = metadata {
            for (k, v) in metadata {
                form_data.push((format!("[metadata][{k}]"), v));
            }
        }
        self.post("subscriptions", &form_data).await
    }

    /// Delete an existing Subscription.
    pub async fn delete_subscription(&self, subscription_id: &SubscriptionId) -> Result<(), Error> {
        self.delete(&format!("subscriptions/{subscription_id}"))
            .await
    }

    /// List up to 10 subscriptions for the specified customer.
    pub async fn list_subscriptions(
        &self,
        customer_id: &CustomerId,
    ) -> Result<Vec<Subscription>, Error> {
        #[derive(Debug, Deserialize)]
        struct SubscriptionList {
            data: Vec<Subscription>,
        }
        let list: SubscriptionList = self
            .get(&format!("customers/{customer_id}/subscriptions?limit=10"))
            .await?;
        Ok(list.data)
    }

    /// Update Subscription with the specified form data.
    pub async fn update_subscription<F: Debug + Serialize>(
        &self,
        subscription_id: &SubscriptionId,
        form_data: &F,
    ) -> Result<Subscription, Error> {
        self.post(&format!("subscriptions/{subscription_id}"), form_data)
            .await
    }
}
