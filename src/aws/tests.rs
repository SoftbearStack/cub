// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(test)]
mod aws_tests {
    use crate::aws::b64::{b64_to_u64, u64_to_b64};
    use crate::aws::translate::{
        braced_names, new_translate_client, to_names, to_numbers, translate_text,
    };
    use crate::common::CubConfig;

    #[test]
    fn b64_tests() {
        println!("Testing b64");
        let n1: u64 = 0;
        let s1 = u64_to_b64(n1);
        let t1 = b64_to_u64(&s1);
        println!("n1 = {} => {} (len {}) => {}", n1, s1, s1.len(), t1);
        let n2: u64 = u64::try_from((1u128 << 64) - 1).unwrap();
        let s2 = u64_to_b64(n2);
        let t2 = b64_to_u64(&s2);
        println!("n1 = {} => {} (len {}) => {}", n2, s2, s2.len(), t2);
    }

    #[tokio::test]
    async fn translate_tests() {
        println!("Testing translate");
        let sample_text = "The cat {name} and the hat {size}";
        let vars = braced_names(sample_text);
        println!("{sample_text} => {vars:?}");
        let a = to_numbers(sample_text, &vars);
        println!("to_number: {a}");
        let b = to_names(&a, &vars);
        println!("to_name: {b}");

        let cub_config = CubConfig::builder()
            .toml_str(
                r#"
                [aws]
                profile = "test_profile"
                "#,
            )
            .build()
            .expect("translate_tests.toml");
        let client = new_translate_client(&cub_config).await;
        let source_language_code = "en";
        let target_language_code = "es";
        let english_text = "The cat {name} and the hat";
        match translate_text(
            &client,
            english_text,
            source_language_code,
            target_language_code,
        )
        .await
        {
            Ok(translated_text) => println!("translated_text={translated_text}"),
            _ => println!("cannot translate"),
        }
    }
}
