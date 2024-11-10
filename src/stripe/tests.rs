// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(test)]
mod stripe_tests {
    use crate::common::CubConfig;
    use crate::stripe::{PriceId, StripeClient};

    fn test_config() -> CubConfig {
        CubConfig::builder()
        .toml_str(
            r#"
                [stripe]
                secret_key = "sk_test_51LcdlTLmirvo61yjaytjXs8YRPPRGdLnLPZLf7v8iukPlBAfm9UL5EPG2COUS85ggptC3lqxl1DNGXrAfCjIpHYw00Oa5xzSAC"
            "#,
        )
        .debug(true)
        .build()
        .expect("stripe_tests.toml")
    }

    #[tokio::test]
    async fn customer_tests() {
        println!("Stripe customer tests");
        let stripe = StripeClient::new(&test_config());
        println!("List products");
        let mut default_price_id: Option<PriceId> = None;
        match stripe.list_products().await {
            Ok(products) => {
                for p in products {
                    if let Some(price_id) = p.default_price {
                        default_price_id = Some(price_id);
                        break;
                    }
                }
            }
            Err(e) => panic!("Error: {e:?}"),
        }
        println!("default_price is {default_price_id:?}");
        println!("Create customer");
        let form_data = [("name", "Mr. Ed"), ("phone", "206-555-1212")];
        let customer = match stripe.create_customer(&form_data).await {
            Ok(customer) => {
                println!("create succeeded: {customer:?}");
                customer
            }
            Err(e) => panic!("Error: {e:?}"),
        };
        println!("Load customer");
        let mut customer = match stripe.load_customer(&customer.id).await {
            Ok(customer) => {
                println!("load succeeded: {customer:?}");
                customer
            }
            Err(e) => panic!("Error: {e:?}"),
        };
        println!("Create payment method");
        let _payment_method_id = match stripe
            .create_card_payment_method(&customer.id, 4242424242424242u64, 12u8, 2025u16, 123u16)
            .await
        {
            Ok(payment_method) => {
                println!(
                    "create payment method for {} succeeded: {payment_method:?}",
                    customer.name.unwrap_or("?".to_string())
                );
                payment_method.id
            }
            Err(e) => panic!("Error: {e:?}"),
        };
        if let Some(price_id) = default_price_id {
            println!("Create subscription");
            match stripe
                .create_subscription(&customer.id, &price_id, 7, None)
                .await
            {
                Ok(subscription) => {
                    println!("subscription created: {subscription:?}");
                }
                Err(e) => panic!("Error: {e:?}"),
            }
        }
        println!("Verify join");
        match stripe.load_customer(&customer.id).await {
            Ok(customer) => {
                println!("verification load succeeded: {customer:?}");
                if customer.default_payment_method.is_none() {
                    panic!("default payment method was not added");
                }
                customer
            }
            Err(e) => panic!("Error: {e:?}"),
        };
        // println!("Save customer");
        customer.name = Some("Mister Edward".to_string());
        // stripe.save_customer(&customer).await
        println!("List customers");
        let _customers = match stripe.list_customers().await {
            Ok(customers) => {
                println!("list succeeded: {customers:?}");
                customers
            }
            Err(e) => panic!("Error: {e:?}"),
        };
        println!("Delete customer");
        match stripe.delete_customer(&customer.id).await {
            Ok(_) => println!("delete succeeded"),
            Err(e) => panic!("Error: {e:?}"),
        }
    }

    #[tokio::test]
    async fn price_tests() {
        println!("Stripe price tests starting");
        let stripe = StripeClient::new(&test_config());
        println!("Load prices");
        let _prices = match stripe.list_prices().await {
            Ok(prices) => {
                println!("load succeeded: {prices:?}");
                prices
            }
            Err(e) => panic!("Error: {e:?}"),
        };
        println!("Stripe price tests completed");
    }

    #[tokio::test]
    async fn product_tests() {
        println!("Stripe product tests starting");
        let stripe = StripeClient::new(&test_config());
        println!("List products");
        let _products = match stripe.list_products().await {
            Ok(products) => {
                println!("list succeeded: {products:?}");
                products
            }
            Err(e) => panic!("Error: {e:?}"),
        };
        println!("Stripe product tests completed");
    }
}
