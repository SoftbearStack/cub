// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::common::{CubConfig, Error};
use aws_config::profile::ProfileFileRegionProvider;
use aws_config::{BehaviorVersion, ConfigLoader, SdkConfig};
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_dynamo::Item;
use std::collections::HashMap;
use std::mem;

/// A convenient alias for Dynamo DB client so consuming code doesn't need to add it to `Cargo.toml`
pub type DynamoDbClient = aws_sdk_dynamodb::Client;

/// Create an AWS config loader with profile and region.
pub fn create_aws_config_loader(cub_config: &CubConfig) -> ConfigLoader {
    #[derive(Deserialize)]
    struct AwsConfig {
        profile: Option<String>,
    }
    #[derive(Deserialize)]
    struct ConfigToml {
        aws: AwsConfig,
    }
    let mut config_loader = aws_config::defaults(BehaviorVersion::v2023_11_09());
    if let Ok(ConfigToml {
        aws: AwsConfig {
            profile: profile_name,
        },
    }) = cub_config.get()
    {
        if let Some(profile_name) = profile_name {
            if cub_config.debug() {
                println!("AWS using profile name {profile_name}");
            }
            let region = ProfileFileRegionProvider::builder()
                .profile_name(&profile_name)
                .build();
            config_loader = config_loader.profile_name(&profile_name).region(region)
        }
    };
    // TODO: let options = Options::from_args();
    config_loader
}

/// Load AWS configuration with profile and region.
pub async fn load_aws_config(config: &CubConfig) -> SdkConfig {
    let config_loader = create_aws_config_loader(config);
    config_loader.load().await
}

/// Creates a Dynamo DB client.
pub async fn new_ddb_client(config: &CubConfig) -> DynamoDbClient {
    let config = load_aws_config(config).await;
    Client::new(&config)
}

/// Creates an item in the specified Dynamo DB table only if its hash key (aka partition
/// key) does not exist.  If the table has a sort key (aka range key), then the (hash key,
/// sort key) tuple must not exist.  (This function does not have a `range_name` parameter
/// because the record put is identified by values in `item` and the Dynamo DB condition
/// merely checks whether said record already exists.)
pub async fn create_ddb_item<I: Serialize>(
    client: &DynamoDbClient,
    item: I,
    table: &'static str,
    hash_name: &'static str,
) -> Result<(), Error> {
    let ser = match serde_dynamo::to_item(item) {
        Ok(ser) => ser,
        Err(e) => return Err(Error::Serde(e)),
    };

    let req = client
        .put_item()
        .table_name(table)
        .expression_attribute_names("#hn", hash_name)
        .condition_expression("attribute_not_exists(#hn)")
        .set_item(Some(ser));

    match req.send().await {
        Err(e) => Err(Error::Dynamo(
            e.into(),
            format!("create_item(t={table}, h={hash_name})"),
        )),
        Ok(_) => Ok(()),
    }
}

/// Deletes an item with the specified hash key, if any, from the specified Dynamo DB table.
pub async fn delete_ddb_item<HK: Serialize>(
    client: &DynamoDbClient,
    table: &'static str,
    hash_name: &'static str,
    hash_value: &HK,
) -> Result<(), Error> {
    let hash_ser = to_dynamo_av(hash_value)?;

    client
        .delete_item()
        .table_name(table)
        .key(hash_name, hash_ser)
        .send()
        .await
        .map_err(|e| Error::Dynamo(e.into(), format!("delete_item(t={table}, h={hash_name})")))?;
    Ok(())
}

/// Deletes an item with the specified hash and range keys, if any, from the specified Dynamo DB table.
pub async fn delete_ddb_ranged_item<HK: Serialize, RK: Serialize>(
    client: &DynamoDbClient,
    table: &'static str,
    hash_name: &'static str,
    hash_value: &HK,
    range_name: &'static str,
    range_value: &RK,
) -> Result<(), Error> {
    let hash_ser = to_dynamo_av(hash_value)?;
    let range_ser = to_dynamo_av(range_value)?;

    client
        .delete_item()
        .table_name(table)
        .key(hash_name, hash_ser)
        .key(range_name, range_ser)
        .send()
        .await
        .map_err(|e| {
            Error::Dynamo(
                e.into(),
                format!("delete_ranged_item(t={table}, h={hash_name}, r={range_name})"),
            )
        })?;
    Ok(())
}

/// Gets the length of the table.
pub async fn describe_ddb_table_length<HK: Serialize>(
    client: &DynamoDbClient,
    table: &'static str,
) -> Result<usize, Error> {
    let output = client
        .describe_table()
        .table_name(table)
        .send()
        .await
        .map_err(|e| Error::Dynamo(e.into(), format!("describe_table(t={table})")))?;
    let len: i64 = output.table().and_then(|d| d.item_count).unwrap_or(0);
    Ok(len.try_into().unwrap_or(0))
}

/// Gets an item with the specified hash key, if any, from the specified Dynamo DB table.
pub async fn get_ddb_item<HK: Serialize, O: DeserializeOwned>(
    client: &DynamoDbClient,
    table: &'static str,
    hash_name: &'static str,
    hash_value: HK,
) -> Result<Option<O>, Error> {
    let hash_ser = to_dynamo_av(hash_value)?;

    let mut get_item_output = match client
        .get_item()
        .consistent_read(true)
        .table_name(table)
        .key(hash_name, hash_ser)
        .send()
        .await
    {
        Ok(output) => output,
        Err(e) => {
            return Err(Error::Dynamo(
                e.into(),
                format!("get_item(t={table}, h={hash_name})"),
            ))
        }
    };

    if let Some(item) = mem::take(&mut get_item_output.item) {
        match serde_dynamo::from_item(item) {
            Err(e) => Err(Error::Serde(e)),
            Ok(de) => Ok(Some(de)),
        }
    } else {
        Ok(None)
    }
}

/// Gets an item with specified hash and range keys, if any, from the specified Dynamo DB table.
pub async fn get_ddb_ranged_item<HK: Serialize, RK: Serialize, O: DeserializeOwned>(
    client: &DynamoDbClient,
    table: &'static str,
    hash_name: &'static str,
    hash_value: HK,
    range_name: &'static str,
    range_value: RK,
) -> Result<Option<O>, Error> {
    let hash_ser = to_dynamo_av(hash_value)?;
    let range_ser = to_dynamo_av(range_value)?;

    let mut get_item_output = match client
        .get_item()
        .consistent_read(true)
        .table_name(table)
        .key(hash_name, hash_ser)
        .key(range_name, range_ser)
        .send()
        .await
    {
        Ok(output) => output,
        Err(e) => {
            return Err(Error::Dynamo(
                e.into(),
                format!("get_ranged_item(t={table}, h={hash_name}, r={range_name})"),
            ))
        }
    };

    if let Some(item) = mem::take(&mut get_item_output.item) {
        match serde_dynamo::from_item(item) {
            Err(e) => Err(Error::Serde(e)),
            Ok(de) => Ok(Some(de)),
        }
    } else {
        Ok(None)
    }
}

async fn query_inner<O: DeserializeOwned>(
    client: &DynamoDbClient,
    table: &'static str,
    hash_name: &'static str,
    hash_value: AttributeValue,
    range_key_bounds: Option<(&'static str, Option<AttributeValue>, Option<AttributeValue>)>,
    last_evaluated_key: Option<HashMap<String, AttributeValue>>,
    ignore_corrupt: bool,
) -> Result<(Vec<O>, Option<HashMap<String, AttributeValue>>), Error> {
    let mut scan = client
        .query()
        .consistent_read(true)
        .table_name(table)
        .expression_attribute_names("#h", hash_name)
        .expression_attribute_values(":hv", hash_value)
        .set_exclusive_start_key(last_evaluated_key);

    if let Some(key_bounds) = range_key_bounds {
        match (key_bounds.1, key_bounds.2) {
            (None, None) => scan = scan.key_condition_expression("#h = :hv"),
            (Some(lo), None) => {
                scan = scan
                    .key_condition_expression("#h = :hv AND #r >= :lo")
                    .expression_attribute_names("#r", key_bounds.0)
                    .expression_attribute_values(":lo", lo)
            }
            (None, Some(hi)) => {
                scan = scan
                    .key_condition_expression("#h = :hv AND #r <= hi")
                    .expression_attribute_names("#r", key_bounds.0)
                    .expression_attribute_values(":hi", hi)
            }
            (Some(lo), Some(hi)) => {
                scan = scan
                    .key_condition_expression("#h = :hv AND #r BETWEEN :lo AND :hi")
                    .expression_attribute_names("#r", key_bounds.0)
                    .expression_attribute_values(":lo", lo)
                    .expression_attribute_values(":hi", hi)
            }
        }
    } else {
        scan = scan.key_condition_expression("#h = :hv");
    }

    let scan_output = match scan.send().await {
        Ok(output) => output,
        Err(e) => {
            return Err(Error::Dynamo(
                e.into(),
                format!("query_inner(t={table}, h={hash_name})"),
            ))
        }
    };

    let mut ret = Vec::new();
    for item in scan_output.items.unwrap_or_default() {
        match serde_dynamo::from_item(item) {
            Err(e) => {
                if !ignore_corrupt {
                    return Err(Error::Serde(e));
                }
            }
            Ok(de) => ret.push(de),
        }
    }
    Ok((ret, scan_output.last_evaluated_key))
}

/// Query and return items from the specified Dynamo DB table.
pub async fn query_ddb<HK: Serialize, O: DeserializeOwned>(
    client: &DynamoDbClient,
    table: &'static str,
    hash_name: &'static str,
    hash_value: HK,
    ignore_corrupt: bool,
) -> Result<Vec<O>, Error> {
    let hash_ser = to_dynamo_av(hash_value)?;

    let mut ret = Vec::new();
    let mut last_evaluated_key = None;
    loop {
        match query_inner(
            client,
            table,
            hash_name,
            hash_ser.clone(),
            None,
            last_evaluated_key,
            ignore_corrupt,
        )
        .await
        {
            Err(e) => return Err(e),
            Ok((mut items, lek)) => {
                ret.append(&mut items);
                last_evaluated_key = lek;

                if last_evaluated_key.is_none() {
                    break;
                }
            }
        }
    }

    Ok(ret)
}

/// Query and return items from the specified Dynamo DB table.
pub async fn query_ddb_hash_range<HK: Serialize, RK: Serialize, O: DeserializeOwned>(
    client: &DynamoDbClient,
    table: &'static str,
    hash_key: (&'static str, HK),
    range_key_bounds: (&'static str, Option<RK>, Option<RK>),
    ignore_corrupt: bool,
) -> Result<Vec<O>, Error> {
    let hash_ser = to_dynamo_av(hash_key.1)?;

    let bounds = (
        range_key_bounds.0,
        if let Some(b) = range_key_bounds.1 {
            Some(to_dynamo_av(b)?)
        } else {
            None
        },
        if let Some(b) = range_key_bounds.2 {
            Some(to_dynamo_av(b)?)
        } else {
            None
        },
    );

    let mut ret = Vec::new();
    let mut last_evaluated_key = None;
    loop {
        match query_inner(
            client,
            table,
            hash_key.0,
            hash_ser.clone(),
            Some(bounds.clone()),
            last_evaluated_key,
            ignore_corrupt,
        )
        .await
        {
            Err(e) => return Err(e),
            Ok((mut items, lek)) => {
                ret.append(&mut items);
                last_evaluated_key = lek;

                if last_evaluated_key.is_none() {
                    break;
                }
            }
        }
    }

    Ok(ret)
}

/// Put an item into the specified Dynamo DB table.
pub async fn put_ddb_item<I: Serialize>(
    client: &DynamoDbClient,
    item: I,
    table: &'static str,
) -> Result<(), Error> {
    let ser = match serde_dynamo::to_item(item) {
        Ok(ser) => ser,
        Err(e) => return Err(Error::Serde(e)),
    };

    let req = client.put_item().table_name(table).set_item(Some(ser));

    match req.send().await {
        Err(e) => Err(Error::Dynamo(e.into(), format!("put_item(t={table})"))),
        Ok(_) => Ok(()),
    }
}

async fn scan_inner<O: DeserializeOwned>(
    client: &DynamoDbClient,
    table: &'static str,
    last_evaluated_key: Option<HashMap<String, AttributeValue>>,
) -> Result<(Vec<O>, Option<HashMap<String, AttributeValue>>), Error> {
    let scan_output = match client
        .scan()
        .consistent_read(true)
        .table_name(table)
        .set_exclusive_start_key(last_evaluated_key)
        .send()
        .await
    {
        Ok(output) => output,
        Err(e) => return Err(Error::Dynamo(e.into(), format!("scan_inner(t={table})"))),
    };

    let mut ret = Vec::new();
    for item in scan_output.items.unwrap_or_default() {
        match serde_dynamo::from_item(item) {
            Err(e) => return Err(Error::Serde(e)),
            Ok(de) => ret.push(de),
        }
    }
    Ok((ret, scan_output.last_evaluated_key))
}

/// Scan and return items from the specified Dynamo DB table.
pub async fn scan_ddb<O: DeserializeOwned>(
    client: &DynamoDbClient,
    table: &'static str,
) -> Result<Vec<O>, Error> {
    let mut ret = Vec::new();
    let mut last_evaluated_key = None;
    loop {
        match scan_inner(client, table, last_evaluated_key).await {
            Err(e) => return Err(e),
            Ok((mut items, lek)) => {
                ret.append(&mut items);
                last_evaluated_key = lek;

                if last_evaluated_key.is_none() {
                    break;
                }
            }
        }
    }

    Ok(ret)
}

/// Packs a Dynamo DB `AttributeValue`.
pub fn to_dynamo_av<T: Serialize>(value: T) -> Result<AttributeValue, Error> {
    serde_dynamo::to_attribute_value(value).map_err(Error::Serde)
}

/// Unpacks a Dynamo DB `AttributeValue::N`.
pub fn to_dynamo_den<T: DeserializeOwned>(s: &str) -> Option<T> {
    serde_dynamo::from_attribute_value(AttributeValue::N(String::from(s))).ok()
}

/// Unpacks a Dynamo DB `AttributeValue::S`.
pub fn to_dynamo_des<T: DeserializeOwned>(s: &str) -> Option<T> {
    serde_dynamo::from_attribute_value(AttributeValue::S(String::from(s))).ok()
}

/// Packs a Dynamo DB item.
pub fn to_dynamo_item<T: Serialize, I: From<Item>>(value: T) -> Result<I, Error> {
    serde_dynamo::to_item(value).map_err(Error::Serde)
}

/// Packs a Dynamo DB `AttributeValue::N` and returns string.
pub fn to_dynamo_sen<T: Serialize>(t: T) -> Option<String> {
    let av: AttributeValue = serde_dynamo::to_attribute_value(t).ok()?;
    if let AttributeValue::N(string) = av {
        Some(string)
    } else {
        None
    }
}

/// Packs a Dynamo DB `AttributeValue::S` and returns string.
pub fn to_dynamo_ses<T: Serialize>(t: T) -> Option<String> {
    let av: AttributeValue = serde_dynamo::to_attribute_value(t).ok()?;
    if let AttributeValue::S(string) = av {
        Some(string)
    } else {
        None
    }
}

/// Update an existing item in the specified Dynamo DB table.
pub async fn update_ddb_item<I: Serialize>(
    client: &DynamoDbClient,
    item: I,
    table: &'static str,
    hash_name: &'static str,
    version_name: &'static str,
    version: usize,
) -> Result<bool, Error> {
    let ser = match serde_dynamo::to_item(item) {
        Ok(ser) => ser,
        Err(e) => return Err(Error::Serde(e)),
    };

    let req = client
        .put_item()
        .table_name(table)
        .expression_attribute_names("#hn", hash_name)
        .expression_attribute_names("#vn", version_name)
        .condition_expression(
            "attribute_exists(#hn) and (attribute_not_exists(#vn) or #vn = :version)",
        )
        .expression_attribute_values(":version", to_dynamo_av(version.saturating_sub(1))?)
        .set_item(Some(ser));

    match req.send().await {
        Err(e) => Err(Error::Dynamo(
            e.into(),
            format!("update_item(t={table}, h={hash_name})"),
        )),
        Ok(_) => Ok(true), // TODO: return false if update failed due to condition.
    }
}
