// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

/// Customer billing address.
mod billing_address;
/// Credit or debit card.
mod charge_card;
/// Customer.
mod customer;
/// Payment method.
mod payment_method;
/// Price.
mod price;
/// Product.
mod product;
/// Stripe resource List.
mod resource_list;
/// Stripe HTTP client.
mod stripe_client;
/// Subscription.
mod subscription;
/// Tests.
mod tests;

pub use self::billing_address::{BillingAddress, BillingDetails};
pub use self::charge_card::{Brand, ChargeCard, CheckResult, Checks, Funding};
pub use self::customer::{Customer, CustomerId};
pub use self::payment_method::{PaymentMethod, PaymentMethodId};
pub use self::price::{Currency, Price, PriceId, PriceType};
pub use self::product::{Product, ProductId};
pub use self::resource_list::StripeResourceList;
pub use self::stripe_client::{new_stripe_client, StripeClient};
pub use self::subscription::{
    AutomaticTax, CancellationDetails, CollectionMethod, Subscription, SubscriptionId,
    SubscriptionItem, SubscriptionItemId, SubscriptionStatus,
};
