// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{to_dynamo_av, DynamoDbClient};
use crate::common::{DynamoError, Error};
use aws_sdk_dynamodb::operation::update_item::builders::UpdateItemFluentBuilder;
use hyper::StatusCode;
use serde::Serialize;
use std::collections::HashSet;

/// Return Dynamo DB update builder.
pub fn ddb_update<T: Serialize>(
    client: &DynamoDbClient,
    table: &'static str,
    hash_name: &'static str,
    hash_value: T,
) -> Result<DynamoUpdateBuilder, Error> {
    let ddb_builder = client
        .update_item()
        .table_name(table)
        .key(hash_name, to_dynamo_av(hash_value)?)
        .condition_expression(&format!("attribute_exists(#{hash_name})"))
        .expression_attribute_names(&format!("#{hash_name}"), hash_name);
    Ok(DynamoUpdateBuilder {
        ddb_builder,
        keys: vec![hash_name].into_iter().collect(),
        removals: Default::default(),
        updates: Default::default(),
    })
}

/// Builder for Dynamo DB update.
pub struct DynamoUpdateBuilder {
    ddb_builder: UpdateItemFluentBuilder,
    keys: HashSet<&'static str>,
    removals: Vec<String>,
    updates: Vec<(String, String)>,
}

impl DynamoUpdateBuilder {
    /// Specify an attribute for the update that will always be set.
    pub fn attribute<T: Serialize>(
        mut self,
        attribute_name: &'static str,
        value: T,
    ) -> Result<Self, Error> {
        if self.keys.contains(attribute_name) {
            return Err(Error::Http(
                StatusCode::FORBIDDEN,
                format!("{attribute_name}: duplicate attribute name"),
            ));
        }

        self.keys.insert(attribute_name);
        let name_key = format!("#{attribute_name}");
        let value_key = format!(":{attribute_name}");
        self.ddb_builder = self
            .ddb_builder
            .expression_attribute_names(&name_key, attribute_name)
            .expression_attribute_values(&value_key, to_dynamo_av(value)?);
        self.updates.push((name_key, value_key));
        Ok(self)
    }

    /// Specify an optional attribute for the update.
    pub fn optional_attribute<T: Serialize>(
        mut self,
        attribute_name: &'static str,
        value: Option<T>,
    ) -> Result<Self, Error> {
        if let Some(value) = value {
            self.attribute(attribute_name, value)
        } else {
            if self.keys.contains(attribute_name) {
                return Err(Error::Http(
                    StatusCode::FORBIDDEN,
                    format!("{attribute_name}: duplicate attribute name"),
                ));
            }
            self.keys.insert(attribute_name);
            let name_key = format!("#{attribute_name}");
            self.ddb_builder = self
                .ddb_builder
                .expression_attribute_names(&name_key, attribute_name);
            self.removals.push(name_key);
            Ok(self)
        }
    }

    /// Start the Dynamo DB update.
    pub async fn send(self) -> Result<String, DynamoError> {
        let updates = if self.updates.is_empty() {
            Default::default()
        } else {
            format!(
                "SET {}",
                self.updates
                    .iter()
                    .map(|(name_key, value_key)| format!("{name_key} = {value_key}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let removals = if self.removals.is_empty() {
            Default::default()
        } else {
            format!("REMOVE {}", self.removals.join(", "))
        };
        let expr = if updates.is_empty() && removals.is_empty() {
            Default::default()
        } else if removals.is_empty() {
            updates
        } else if updates.is_empty() {
            removals
        } else {
            format!("{updates} {removals}")
        };
        if !expr.is_empty() {
            self.ddb_builder
                .update_expression(&expr)
                .send()
                .await
                .map_err(|e| {
                    let e: DynamoError = e.into();
                    e
                })?;
        }
        Ok(expr)
    }

    /// Specify an attribute that wont be set if it equals its default value.
    pub fn skippable_attribute<T: Default + PartialEq + Serialize>(
        self,
        attribute_name: &'static str,
        value: T,
    ) -> Result<Self, Error> {
        let value: Option<T> = if value != T::default() {
            Some(value)
        } else {
            None
        };
        self.optional_attribute(attribute_name, value)
    }
}
