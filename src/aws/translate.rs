// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use super::load_aws_config;
use crate::common::{CubConfig, Error};
use aws_sdk_translate::Client;
use std::collections::HashSet;

/// A convenient alias for translate client so consuming code doesn't need to add it to `Cargo.toml`
pub type TranslateClient = aws_sdk_translate::Client;

/// Creates a Translate client.
pub async fn new_translate_client(cub_config: &CubConfig) -> TranslateClient {
    let aws_config = load_aws_config(cub_config).await;
    Client::new(&aws_config)
}

/// Returns braced names which appear in a string.  For example, `{me}`.
pub(crate) fn braced_names(source_text: &str) -> Vec<String> {
    let mut name_hash: HashSet<String> = HashSet::new();
    let mut name = Vec::new();
    let mut parsing_name = false;
    for ch in source_text.chars() {
        match ch {
            '{' if !parsing_name => parsing_name = true,
            '{' if parsing_name => parsing_name = false,
            '}' if parsing_name => {
                if !name.is_empty() {
                    let v = name.iter().collect();
                    name_hash.insert(v);
                    name.clear();
                }
                parsing_name = false;
            }
            _ => {
                if parsing_name {
                    name.push(ch);
                }
            }
        }
    }
    name_hash.into_iter().collect()
}

/// Returns true if the target has all of the variables present in the source.
pub fn braces_valid(source_text: &str, target_text: &str) -> bool {
    let target_names: HashSet<_> = braced_names(target_text).into_iter().collect();
    !braced_names(source_text)
        .into_iter()
        .any(|name| !target_names.contains(&name))
}

/// Replaces braced numbers with braced names in a string.
pub(crate) fn to_names(source_text: &str, vars: &Vec<String>) -> String {
    let mut result = source_text.to_owned();
    for (i, name) in vars.iter().enumerate() {
        let name: &str = &name;
        result = result.replace(&format!("{{{i}}}"), &format!("{{{name}}}"));
    }
    result
}

/// Replaces braced braced names with braced numbers in a string.
pub(crate) fn to_numbers(source_text: &str, vars: &Vec<String>) -> String {
    let mut result = source_text.to_owned();
    for (i, name) in vars.iter().enumerate() {
        let name: &str = &name;
        result = result.replace(&format!("{{{name}}}"), &format!("{{{i}}}"));
    }
    result
}

/// Translates text from one language to another.
pub async fn translate_text(
    client: &TranslateClient,
    source_text: &str,
    source_language_code: &str,
    target_language_code: &str,
) -> Result<String, Error> {
    let vars = braced_names(source_text);
    let source_text = to_numbers(source_text, &vars);
    let output = client
        .translate_text()
        .source_language_code(source_language_code.to_owned())
        .target_language_code(target_language_code.to_owned())
        .text(&source_text)
        .send()
        .await
        .map_err(|e| Error::Anyhow(e.into(), format!("translate_text({source_text})")))?;
    let target_text = output.translated_text();
    let target_text = to_names(target_text, &vars);
    Ok(target_text)
}
