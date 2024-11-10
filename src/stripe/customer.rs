// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{BillingAddress, Currency, PaymentMethod, PaymentMethodId, StripeClient, Subscription};
use crate::common::Error;
use crate::impl_wrapper_str;
use crate::serde_utils::is_default;
use crate::time_id::NonZeroUnixSeconds;
use core::fmt::Debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Customer ID.
pub struct CustomerId(pub String);
impl_wrapper_str!(CustomerId);

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Customer.
pub struct Customer {
    /// Unique identifier for the customer.
    pub id: CustomerId,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The customer's billing address.
    pub address: Option<BillingAddress>,

    /// Customer balance in cents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Date/time record was created.
    pub created: Option<NonZeroUnixSeconds>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// 3-letter currency designation for recurring billing, e.g. Currency::USD.
    pub currency: Option<Currency>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The customer's default payment method, if any.
    //  (Obtained by a second query from ID in invoice_settings.)
    pub default_payment_method: Option<PaymentMethod>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// Customers aren't actually deleted but are flagged as such.
    pub deleted: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Customer description.
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The customer's email address.
    pub email: Option<String>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// Live mode vs test mode.
    pub livemode: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Invoice settings such as the default payment method.
    pub invoice_settings: Option<InvoiceSettings>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The customer's full name.
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The customer's phone number.
    pub phone: Option<String>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// Application specific metadata.
    pub metadata: HashMap<String, String>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The customer's subscriptions, if any.
    //  (Obtained by a second query.)
    pub subscriptions: Vec<Subscription>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Invoice settings such as the default payment method.
pub struct InvoiceSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The customer's default payment method.
    default_payment_method: Option<PaymentMethodId>,
}

impl StripeClient {
    /// Create a Customer with the specified form data.
    pub async fn create_customer<F: Debug + Serialize>(
        &self,
        form_data: &F,
    ) -> Result<Customer, Error> {
        self.post("customers", form_data).await
    }

    /// Delete an existing Customer.
    pub async fn delete_customer(&self, customer_id: &CustomerId) -> Result<(), Error> {
        self.delete(&format!("customers/{customer_id}")).await
    }

    /// Join to lists of payment methods, subscriptions, etc.
    async fn join_to_lists(&self, customer: &mut Customer) -> Result<(), Error> {
        // Join with default payment method.
        if let Some(default_payment_method_id) = customer
            .invoice_settings
            .as_ref()
            .map(|i| i.default_payment_method.clone())
            .flatten()
        {
            let pm = self
                .load_payment_method(&customer.id, &default_payment_method_id)
                .await?;
            customer.default_payment_method = Some(pm);
        }
        // Join with subscriptions
        for s in self.list_subscriptions(&customer.id).await? {
            customer.subscriptions.push(s);
        }
        Ok(())
    }

    /// List up to 10 customers.
    pub async fn list_customers(&self) -> Result<Vec<Customer>, Error> {
        #[derive(Debug, Deserialize)]
        struct CustomerList {
            data: Vec<Customer>,
        }
        let mut list: CustomerList = self.get("customers?limit=10").await?;
        list.data.retain(|p| !p.deleted);
        for customer in &mut list.data {
            self.join_to_lists(customer).await?;
        }

        Ok(list.data)
    }

    /// Load an existing Customer.
    pub async fn load_customer(&self, customer_id: &CustomerId) -> Result<Customer, Error> {
        let mut customer: Customer = self.get(&format!("customers/{customer_id}")).await?;
        self.join_to_lists(&mut customer).await?;
        Ok(customer)
    }

    /// Update Customer with the specified form data.  Any scalar parameters not provided
    /// will be left unchanged, but if any part of address is modified then the entire
    /// address must be provided.
    pub async fn update_customer<F: Debug + Serialize>(
        &self,
        customer_id: &CustomerId,
        form_data: &F,
    ) -> Result<Customer, Error> {
        let mut customer = self
            .post(&format!("customers/{customer_id}"), form_data)
            .await?;
        self.join_to_lists(&mut customer).await?;
        Ok(customer)
    }
}
