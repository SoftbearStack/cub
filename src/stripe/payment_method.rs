// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use super::{BillingDetails, ChargeCard, CustomerId, StripeClient};
use crate::common::Error;
use crate::impl_wrapper_str;
use crate::serde_utils::is_default;
use crate::time_id::NonZeroUnixSeconds;
use core::fmt::Debug;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Payment Source ID.
pub struct PaymentMethodId(pub String);
impl_wrapper_str!(PaymentMethodId);

/// Payment method, e.g. CC.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct PaymentMethod {
    /// Unique identifier for the object.
    pub id: PaymentMethodId,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Customer billing details (address, email, name, and phone).
    pub billing_details: Option<BillingDetails>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Credit or debit card.
    pub card: Option<ChargeCard>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Date/time record was created.
    pub created: Option<NonZeroUnixSeconds>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Customer that owns this payment method, if attached.
    pub customer: Option<CustomerId>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// Live mode vs test mode.
    pub livemode: bool,

    #[serde(rename = "type")]
    /// Whether the credit or debit card was physically present (for online transactions, it is not).
    pub payment_method_type: PaymentMethodType,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
/// Whether the credit or debit card was physically present (for online transactions, it is not).
pub enum PaymentMethodType {
    /// Charge card was not physically present.
    Card,
    /// Card was physically present.
    CardPresent,
}

impl StripeClient {
    /// Create credit or debit card payment method.
    pub async fn create_card_payment_method(
        &self,
        customer_id: &CustomerId,
        card_number: u64,
        exp_month: u8,
        exp_year: u16,
        card_cvc: u16,
    ) -> Result<PaymentMethod, Error> {
        let cc_form_data = [
            ("type", format!("card")),
            ("card[number]", format!("{card_number}")),
            ("card[exp_month]", format!("{exp_month}")),
            ("card[exp_year]", format!("{exp_year}")),
            ("card[cvc]", format!("{card_cvc}")),
        ];
        let PaymentMethod {
            id: payment_method_id,
            ..
        } = self.post("payment_methods", &cc_form_data).await?;
        let customer_form_data = [("customer", format!("{customer_id}"))];
        let payment_method: PaymentMethod = self
            .post(
                &format!("payment_methods/{payment_method_id}/attach"),
                &customer_form_data,
            )
            .await?;
        // The new payment method becomes the default for the customer.
        let form_data = [(
            "invoice_settings[default_payment_method]",
            payment_method.id.to_string(),
        )];
        self.update_customer(customer_id, &form_data).await?;
        Ok(payment_method)
    }

    /// Delete (detach) an existing payment method.
    pub async fn delete_payment_method(
        &self,
        payment_method_id: &PaymentMethodId,
    ) -> Result<(), Error> {
        let form_data: &[(&str, &str)] = &[];
        self.post(
            &format!("payment_methods/{payment_method_id}/detatch"),
            &form_data,
        )
        .await
    }

    /// Load an existing payment method.
    pub async fn load_payment_method(
        &self,
        customer_id: &CustomerId,
        payment_method_id: &PaymentMethodId,
    ) -> Result<PaymentMethod, Error> {
        Ok(self
            .get(&format!(
                "customers/{customer_id}/payment_methods/{payment_method_id}"
            ))
            .await?)
    }

    /// List card up to 10 payment methods for the specified customer.
    pub async fn list_card_payment_methods(
        &self,
        customer_id: &CustomerId,
    ) -> Result<Vec<PaymentMethod>, Error> {
        #[derive(Debug, Deserialize)]
        struct PaymentMethodList {
            data: Vec<PaymentMethod>,
        }
        let list: PaymentMethodList = self
            .get(&format!(
                "customers/{customer_id}/payment_methods?type=card&limit=10"
            ))
            .await?;
        Ok(list.data)
    }

    /// Update credit or debit card payment method.  It is only possible to
    /// update the expiration date, not the card[number] or card[cvc].
    pub async fn update_card_payment_method(
        &self,
        payment_method_id: &PaymentMethodId,
        exp_month: Option<u8>,
        exp_year: Option<u16>,
    ) -> Result<PaymentMethod, Error> {
        let mut form_data: Vec<(String, String)> = Vec::new();
        if let Some(exp_month) = exp_month {
            form_data.push(("card[exp_month]".to_string(), format!("{exp_month}")));
        }
        if let Some(exp_year) = exp_year {
            form_data.push(("card[exp_year]".to_string(), format!("{exp_year}")));
        }
        if form_data.is_empty() {
            Err(Error::Http(
                StatusCode::PRECONDITION_FAILED,
                "no CC payment update".to_string(),
            ))
        } else {
            self.post(&format!("payment_methods/{payment_method_id}"), &form_data)
                .await
        }
    }
}
