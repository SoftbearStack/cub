// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

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

/// A very simple visitor for `String` a commonly used type.
/// `&str` visitor
pub struct StrVisitor;

impl<'de> Visitor<'de> for StrVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a str")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(String::from(value))
    }
}

/// Generic visitor which supports a variety of types.
///
/// ```rust
/// use serde::de::Deserializer;
/// use cub::TypedVisitor;
///
/// let mut de = serde_json::Deserializer::from_str("10");
/// let result = de.deserialize_u8(TypedVisitor::<u8>::default());
/// assert_eq!(result.unwrap(), 10);
///
/// let mut de = serde_json::Deserializer::from_str("100000");
/// let result = de.deserialize_i16(TypedVisitor::<i16>::default());
/// assert!(result.is_err());
/// ```
pub struct TypedVisitor<T>(PhantomData<T>);

impl<T> Default for TypedVisitor<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

macro_rules! visit_exact {
    ($t: ident => $visitor: ident) => {
        fn $visitor<E>(self, value: $t) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }
    };
}

macro_rules! visit_try_into_overflow {
    ($exact_visitor: ident, $($t: ident => $visitor: ident),*) => {
        $(
            fn $visitor<E>(self, value: $t) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.$exact_visitor(value.try_into().map_err(|_| serde::de::Error::custom("overflow"))?)
            }
        )*
    };
}

macro_rules! visit_ints {
    ($($t: ident => $visitor: ident),*) => {
        $(
            impl<'de> Visitor<'de> for TypedVisitor<$t> {
                type Value = $t;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str(concat!("a ", stringify!($t)))
                }

                visit_exact!($t => $visitor);
                visit_try_into_overflow!($visitor, u64 => visit_u64, i64 => visit_i64, u128 => visit_u128, i128 => visit_i128);
            }
        )*
    }
}

visit_ints!(u8 => visit_u8, i8 => visit_i8, u16 => visit_u16, i16 => visit_i16, u32 => visit_u32, i32 => visit_i32);

macro_rules! visit_float {
    ($float: ident => $visitor: ident) => {
        type Value = $float;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a float")
        }

        fn $visitor<E>(self, value: $float) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.$visitor(value as $float)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.$visitor(value as $float)
        }

        fn visit_i128<E>(self, value: i128) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.$visitor(value as $float)
        }

        fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.$visitor(value as $float)
        }
    };
}

impl<'de> Visitor<'de> for TypedVisitor<f32> {
    visit_float!(f32 => visit_f32);

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_f32(value as f32)
    }
}

impl<'de> Visitor<'de> for TypedVisitor<f64> {
    visit_float!(f64 => visit_f64);

    fn visit_f32<E>(self, value: f32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_f64(value as f64)
    }
}

impl<'de> Visitor<'de> for TypedVisitor<Vec<u8>> {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("bytes")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_byte_buf(Vec::from(value))
    }

    fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value)
    }
}

impl<'de> Visitor<'de> for TypedVisitor<String> {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(String::from(value))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value)
    }
}
