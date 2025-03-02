// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use core::marker::PhantomData;
use serde::de;
use serde::de::Visitor;
use std::fmt::{self, Display};
use std::str::FromStr;

/// Implement `Serialize` and `Deserialize`.  For example:
///     serde_str!(Foo);
#[macro_export]
macro_rules! serde_str {
    ($id:ident) => {
        impl Serialize for $id {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.collect_str(self)
            }
        }

        impl<'de> Deserialize<'de> for $id {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_str(FromStrVisitor::<Self>::default())
            }
        }
    };
}

/// Deserializes any type that implements `FromStr`.
pub struct FromStrVisitor<T>(PhantomData<T>);

impl<T> Default for FromStrVisitor<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<'de, T: FromStr<Err: Display>> Visitor<'de> for FromStrVisitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a str")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::from_str(value).map_err(|e| serde::de::Error::custom(e))
    }
}
