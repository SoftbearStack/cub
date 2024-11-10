// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::UnixMillis;
use crate::common::Error;
use crate::{
    impl_wrapper_display, impl_wrapper_display_from_str, impl_wrapper_from_str, impl_wrapper_nz,
};
use rand::Rng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::num::{NonZeroU16, NonZeroU32, NonZeroU64};
use std::str::FromStr;

/// A 16-bit ID.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ID16(pub NonZeroU16);

/// A 32-bit ID.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ID32(pub NonZeroU32);
impl_wrapper_display_from_str!(ID32, NonZeroU32);
impl_wrapper_nz!(ID32, NonZeroU32, u32);

/// A 64-bit ID with optional timestamp.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ID64<const N: usize>(pub NonZeroU64);

impl ID16 {
    /// Generates a new ID.
    #[allow(unused)]
    pub fn generate() -> Self {
        Self(rand::thread_rng().gen())
    }
}

impl Into<NonZeroU16> for ID16 {
    fn into(self) -> NonZeroU16 {
        self.0
    }
}

impl ID32 {
    /// Generates a new ID.
    pub fn generate() -> Self {
        Self(rand::thread_rng().gen())
    }
}

impl<const DAY_BITS: usize> ID64<DAY_BITS> {
    /// Generates a 64-bit ID suitable for a search endpoint with "day" offset `d`
    /// and "random" offset `r` (`r` is typically set to `1` for searches).
    pub fn endpoint(day: u64, r: NonZeroU64) -> Self {
        if DAY_BITS != 0 && DAY_BITS <= 64 {
            let msb = day.wrapping_shl((64 - DAY_BITS).try_into().unwrap());
            let lsb = r.get() & ((1 << (64 - DAY_BITS)) - 1);
            debug_assert!(msb & lsb == 0);
            Self(NonZeroU64::new(msb | lsb).unwrap_or(NonZeroU64::new(1).unwrap()))
        } else {
            Self(r)
        }
    }

    /// Generates a random 64-bit ID which includes timestamp.
    /// Generates a new ID with optional timestamp.  The timestamp resolution
    /// is in days.  Therefore, a 10-bit timestamp codes 1024 days or 2.8 years.
    ///
    /// # Example
    /// `ID64::<10>::generate()`
    pub fn generate() -> Self {
        Self::endpoint(get_unix_day(), rand::thread_rng().gen())
    }
}

impl<const DAY_BITS: usize> Display for ID64<DAY_BITS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl<const DAY_BITS: usize> FromStr for ID64<DAY_BITS> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let n: NonZeroU64 = s
            .parse()
            .map_err(|_| Error::String(format!("{s}: not an ID")))?;
        Ok(Self(n))
    }
}

impl<const DAY_BITS: usize> Into<NonZeroU64> for ID64<DAY_BITS> {
    fn into(self) -> NonZeroU64 {
        self.0
    }
}

impl<const DAY_BITS: usize> Into<String> for ID64<DAY_BITS> {
    fn into(self) -> String {
        self.0.to_string()
    }
}

mod id64_serde {
    use super::*;
    use serde::{Deserialize, Serialize};

    impl<const DAY_BITS: usize> Serialize for ID64<DAY_BITS> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.0.serialize(serializer)
        }
    }

    impl<'de, const DAY_BITS: usize> Deserialize<'de> for ID64<DAY_BITS> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            {
                Deserialize::deserialize(deserializer).map(ID64::<DAY_BITS>)
            }
        }
    }
}

/// Gets value that increments by 1 every 24 hours.
fn get_unix_day() -> u64 {
    let unix_millis: u64 = UnixMillis::now().try_into().unwrap_or_default();
    (unix_millis / (24 * 60 * 60 * 1000)) as u64
}

#[cfg(test)]
mod tests {
    use crate::time_id::ID64;

    #[test]
    fn test_64() {
        let i = ID64::<10>::generate();
        println!("i = {:?}", i);
    }
}
