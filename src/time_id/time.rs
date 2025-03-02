// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

#[cfg(feature = "chrono")]
use crate::common::Error;
use crate::{impl_wrapper_from_str, impl_wrapper_int, impl_wrapper_nz};
#[cfg(feature = "chrono")]
use chrono::offset::LocalResult;
#[cfg(feature = "chrono")]
use chrono::{DateTime, Datelike, Local, NaiveDate, TimeZone, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::num::NonZeroU64;
use std::time::{SystemTime, UNIX_EPOCH};

/// A Unix date/time which contains the number of non leap milliseconds since 1970.
/// `Option<NonZeroUnixMillis>` is more memory effient than `Option<UnixMillis>`.
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
pub struct NonZeroUnixMillis(pub NonZeroU64);
impl_wrapper_nz!(NonZeroUnixMillis, NonZeroU64, u64);
impl_wrapper_from_str!(NonZeroUnixMillis, NonZeroU64);

impl NonZeroUnixMillis {
    /// Creates a `NonZeroUnixMillis` with the current date and time.
    pub fn now() -> Self {
        Self::new()
    }
}

impl UnixTime for NonZeroUnixMillis {
    /// Maximum `NonZeroUnixMillis`.
    const MAX: NonZeroUnixMillis = NonZeroUnixMillis(NonZeroU64::MAX);
    /// Minimum `NonZeroUnixMillis`.
    const MIN: NonZeroUnixMillis = NonZeroUnixMillis(NonZeroU64::MIN);

    fn from_i64(value: i64) -> Self {
        value
            .try_into()
            .ok()
            .and_then(|unsigned_value| NonZeroU64::new(unsigned_value))
            .map(|nonzero_value| Self(nonzero_value))
            .unwrap_or(Self::MIN)
    }

    fn to_i64(&self) -> i64 {
        self.0.get().try_into().unwrap_or(i64::MAX)
    }
}

impl Display for NonZeroUnixMillis {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        #[cfg(feature = "chrono")]
        if f.alternate() {
            return f.write_str(&self.to_default_format());
        }

        Display::fmt(&self.0, f)
    }
}

impl From<NonZeroUnixSeconds> for NonZeroUnixMillis {
    fn from(value: NonZeroUnixSeconds) -> Self {
        Self::from_i64(value.to_i64())
    }
}

impl From<UnixMillis> for NonZeroUnixMillis {
    fn from(value: UnixMillis) -> Self {
        Self::from_i64(value.to_i64())
    }
}

/// A Unix date/time which contains the number of non leap seconds since 1970.
/// `Option<NonZeroUnixSeconds>` is convient for certain expiration dates.
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
pub struct NonZeroUnixSeconds(pub NonZeroU64);
impl_wrapper_nz!(NonZeroUnixSeconds, NonZeroU64, u64);
impl_wrapper_from_str!(NonZeroUnixSeconds, NonZeroU64);

impl NonZeroUnixSeconds {
    /// Creates a `NonZeroUnixSeconds` with the current date and time.
    pub fn now() -> Self {
        Self::new()
    }
}

impl UnixTime for NonZeroUnixSeconds {
    /// Maximum `NonZeroUnixSeconds`.
    const MAX: NonZeroUnixSeconds = NonZeroUnixSeconds(NonZeroU64::MAX);
    /// Minimum `NonZeroUnixSeconds`.
    const MIN: NonZeroUnixSeconds = NonZeroUnixSeconds(NonZeroU64::MIN);

    fn from_i64(value: i64) -> Self {
        (value / Self::MILLIS_PER_SECOND as i64)
            .try_into()
            .ok()
            .and_then(|unsigned_value| NonZeroU64::new(unsigned_value))
            .map(|nonzero_value| Self(nonzero_value))
            .unwrap_or(Self::MIN)
    }

    fn to_i64(&self) -> i64 {
        self.0
            .get()
            .try_into()
            .map(|secs: i64| secs * Self::MILLIS_PER_SECOND as i64)
            .unwrap_or(i64::MAX)
    }
}

impl Display for NonZeroUnixSeconds {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        #[cfg(feature = "chrono")]
        if f.alternate() {
            return f.write_str(&self.to_default_format());
        }

        Display::fmt(&self.0, f)
    }
}

impl From<NonZeroUnixMillis> for NonZeroUnixSeconds {
    fn from(value: NonZeroUnixMillis) -> Self {
        Self::from_i64(value.to_i64())
    }
}

impl From<UnixMillis> for NonZeroUnixSeconds {
    fn from(value: UnixMillis) -> Self {
        Self::from_i64(value.to_i64())
    }
}

/// A Unix date/time which contains the number of non leap milliseconds since (or before) 1970.
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize)]
pub struct UnixMillis(pub i64);
impl_wrapper_int!(UnixMillis, i64, u64);
impl_wrapper_from_str!(UnixMillis, NonZeroU64);

impl UnixMillis {
    /// Creates a `UnixMillis` with the current date and time.
    pub fn now() -> Self {
        Self::new()
    }
}

impl UnixTime for UnixMillis {
    /// Maximum `UnixMillis`.
    const MAX: UnixMillis = UnixMillis(i64::MAX);
    /// Minimum `UnixMillis`.
    const MIN: UnixMillis = UnixMillis(i64::MIN);

    fn from_i64(value: i64) -> Self {
        Self(value)
    }

    fn to_i64(&self) -> i64 {
        self.0
    }
}

impl Display for UnixMillis {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        #[cfg(feature = "chrono")]
        if f.alternate() {
            return f.write_str(&self.to_default_format());
        }

        Display::fmt(&self.0, f)
    }
}

impl From<NonZeroUnixMillis> for UnixMillis {
    fn from(value: NonZeroUnixMillis) -> Self {
        Self::from_i64(value.to_i64())
    }
}

impl From<NonZeroUnixSeconds> for UnixMillis {
    fn from(value: NonZeroUnixSeconds) -> Self {
        Self::from_i64(value.to_i64())
    }
}

/// Convenient time arithmetic.
pub trait UnixTime: Sized + Clone {
    /// Maximum time supported by notation.
    const MAX: Self;
    /// Minimum time supported by notation.
    const MIN: Self;

    /// Milliseconds per second.
    const MILLIS_PER_SECOND: u64 = 1000;
    /// Milliseconds per minute.
    const MILLIS_PER_MINUTE: u64 = 60 * Self::MILLIS_PER_SECOND;
    /// Milliseconds per hour.
    const MILLIS_PER_HOUR: u64 = 60 * Self::MILLIS_PER_MINUTE;
    /// Milliseconds per day.
    const MILLIS_PER_DAY: u64 = 24 * Self::MILLIS_PER_HOUR;
    /// Milliseconds per week.
    const MILLIS_PER_WEEK: u64 = 7 * Self::MILLIS_PER_DAY;

    /// Creates a `UnixTime` with the current date and time.
    fn new() -> Self {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time too low");
        Self::from_i64(
            duration
                .as_millis()
                .try_into()
                .expect("system time too high"),
        )
    }

    /// Adds days to a Unix date/time and returns the result.
    fn add_days(&self, d: u64) -> Self {
        self.add_millis(d * Self::MILLIS_PER_DAY)
    }

    /// Adds hours to a Unix date/time and returns the result.
    fn add_hours(&self, h: u64) -> Self {
        self.add_millis(h * Self::MILLIS_PER_HOUR)
    }

    /// Adds milliseconds to a Unix date/time and returns the result.
    fn add_millis(&self, m: u64) -> Self {
        TryInto::<i64>::try_into(m)
            .map(|m| self.add_signed_millis(m))
            .unwrap_or(Self::MAX)
    }

    /// Adds minutes to a Unix date/time and returns the result.
    fn add_minutes(&self, m: u64) -> Self {
        self.add_millis(m * Self::MILLIS_PER_MINUTE)
    }

    /// Adds seconds to a Unix date/time and returns the result.
    fn add_seconds(&self, s: u64) -> Self {
        self.add_millis(s * Self::MILLIS_PER_SECOND)
    }

    /// Adds (or subtracts) hours to (or from) a Unix date/time and returns the result.
    fn add_signed_hours(&self, h: i64) -> Self {
        self.add_signed_millis(h * Self::MILLIS_PER_HOUR as i64)
    }

    /// Adds (or subtracts) millis to (or from) a Unix date/time and returns the result.
    fn add_signed_millis(&self, m: i64) -> Self {
        Self::from_i64(self.to_i64().saturating_add(m))
    }

    /// Adds (or subtracts) minutes to (or from) a Unix date/time and returns the result.
    fn add_signed_minutes(&self, m: i64) -> Self {
        self.add_signed_millis(m * Self::MILLIS_PER_MINUTE as i64)
    }

    /// Adds (or subtracts) seconds to (or from) a Unix date/time and returns the result.
    fn add_signed_seconds(&self, s: i64) -> Self {
        self.add_signed_millis(s * Self::MILLIS_PER_SECOND as i64)
    }

    /// Adds (or subtracts) weeks to (or from) a Unix date/time and returns the result.
    fn add_signed_weeks(&self, w: i64) -> Self {
        self.add_signed_millis(w * Self::MILLIS_PER_WEEK as i64)
    }

    /// Adds weeks to a Unix date/time and returns the result.
    fn add_weeks(&self, w: u64) -> Self {
        self.add_millis(w * Self::MILLIS_PER_WEEK)
    }

    /// Day number from 1 to 31.
    #[cfg(feature = "chrono")]
    fn day(&self) -> u32 {
        self.to_date_time_utc().day0() + 1
    }

    /// Returns the number of days since the specified Unix date/time.
    fn days_since(&self, unix_time: impl UnixTime) -> u64 {
        self.millis_since(unix_time) / Self::MILLIS_PER_DAY
    }

    /// Returns the number of days which have elapsed since the 1970 epoch.
    fn days_since_epoch(&self) -> u32 {
        (self.to_i64() as u64 / Self::MILLIS_PER_DAY)
            .try_into()
            .unwrap_or_default()
    }

    /// Returns the date/time rounded down to days.  (That is, date/time of the
    /// midnight which precedes the specified time.)
    fn floor_days(&self) -> Self {
        // This can be accomplished via simple math because UTC has no TZ.
        Self::from_i64((self.to_i64() / Self::MILLIS_PER_DAY as i64) * Self::MILLIS_PER_DAY as i64)
    }

    /// Returns the date/time rounded down to hours.
    fn floor_hours(&self) -> Self {
        Self::from_i64(
            (self.to_i64() / Self::MILLIS_PER_HOUR as i64) * Self::MILLIS_PER_HOUR as i64,
        )
    }

    /// Returns the date/time rounded down to minutes.
    fn floor_minutes(&self) -> Self {
        Self::from_i64(
            (self.to_i64() / Self::MILLIS_PER_MINUTE as i64) * Self::MILLIS_PER_MINUTE as i64,
        )
    }

    /// Returns the date/time rounded down to seconds.
    fn floor_seconds(&self) -> Self {
        Self::from_i64(
            (self.to_i64() / Self::MILLIS_PER_SECOND as i64) * Self::MILLIS_PER_SECOND as i64,
        )
    }

    /// Returns time corresponding to i64.
    fn from_i64(value: i64) -> Self;

    /// Format `UnixMillis` as string.
    #[cfg(feature = "chrono")]
    fn format(&self, fmt: &str) -> String {
        self.to_date_time_utc().format(fmt).to_string()
    }

    /// Create a `UnixMillis` from YMD HMS.
    #[cfg(feature = "chrono")]
    fn from_ymdhms(
        year: u32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> Result<Self, Error> {
        let year = year
            .try_into()
            .map_err(|e| Error::String(format!("{e:?}")))?;
        let nd = match NaiveDate::from_ymd_opt(year, month, day) {
            Some(nd) => Ok(nd),
            _ => Err(Error::String(format!("{year}-{month}-{day}"))),
        }?;
        let ndt = match nd.and_hms_opt(hour, minute, second) {
            Some(ndt) => Ok(ndt),
            _ => Err(Error::String(format!("{hour}:{minute}:{second}"))),
        }?;
        let dt: DateTime<Utc> = Utc::from_utc_datetime(&Utc, &ndt);
        Ok(Self::from_i64(dt.timestamp_millis()))
    }

    /// Returns the number of hours since the specified Unix date/time.
    fn hours_since(&self, unix_time: impl UnixTime) -> u64 {
        self.millis_since(unix_time) / Self::MILLIS_PER_HOUR
    }

    /// Returns the milliseconds the specified Unix date/time.
    fn millis_since(&self, unix_time: impl UnixTime) -> u64 {
        self.to_i64()
            .saturating_sub(unix_time.to_i64())
            .try_into()
            .unwrap_or(0)
    }

    /// Returns the minutes since the specified Unix date/time.
    fn minutes_since(&self, unix_time: impl UnixTime) -> u64 {
        self.millis_since(unix_time) / Self::MILLIS_PER_MINUTE
    }

    /// Month number from 1 (Jan) to 12 (Dec).
    #[cfg(feature = "chrono")]
    fn month(&self) -> u32 {
        self.to_date_time_utc().month0() + 1
    }

    /// Subtracts seconds from a Unix date/time and returns the result.
    fn sub_days(&self, days: u64) -> Self {
        self.sub_millis(days * Self::MILLIS_PER_DAY)
    }

    /// Subtracts hours from a Unix date/time and returns the result.
    fn sub_hours(&self, hours: u64) -> Self {
        self.sub_millis(hours * Self::MILLIS_PER_HOUR)
    }

    /// Subtracts milliseconds from a Unix date/time and returns the result.
    fn sub_millis(&self, millis: u64) -> Self {
        TryInto::<i64>::try_into(millis)
            .map(|millis| Self::from_i64(self.to_i64().saturating_sub(millis)))
            .unwrap_or(self.clone())
    }

    /// Subtracts minutes from a Unix date/time and returns the result.
    fn sub_minutes(&self, minutes: u64) -> Self {
        self.sub_millis(minutes * Self::MILLIS_PER_MINUTE)
    }

    /// Subtracts seconds from a Unix date/time and returns the result.
    fn sub_seconds(&self, minutes: u64) -> Self {
        self.sub_millis(minutes * Self::MILLIS_PER_SECOND)
    }

    /// Subtracts weeks from a Unix date/time and returns the result.
    fn sub_weeks(&self, minutes: u64) -> Self {
        self.sub_millis(minutes * Self::MILLIS_PER_WEEK)
    }

    /// Returns i64 corresponding to time.
    #[cfg(feature = "chrono")]
    fn to_date_time_utc(&self) -> DateTime<Utc> {
        match Utc.timestamp_millis_opt(self.to_i64()) {
            LocalResult::Single(dt) => dt,
            // Assume invalid `UnixMillis` never happens, but if it does don't panic.
            _ => Local::now().into(),
        }
    }

    /// Returns a reasonable string representation of the time.
    #[cfg(feature = "chrono")]
    fn to_default_format(&self) -> String {
        self.format("%Y-%m-%d %H:%M")
    }

    /// Returns i64 corresponding to time.
    fn to_i64(&self) -> i64;

    /// Returns the number of weeks since the specified Unix date/time.
    fn weeks_since(&self, unix_time: impl UnixTime) -> u64 {
        self.millis_since(unix_time) / Self::MILLIS_PER_WEEK
    }

    /// Year, e.g. 2024.
    #[cfg(feature = "chrono")]
    fn year(&self) -> u32 {
        let (_ad, year) = self.to_date_time_utc().year_ce();
        year
    }

    /// Convert a `UnixMillis` into YMD HMS.
    #[cfg(feature = "chrono")]
    fn ymdhms(&self) -> (u32, u32, u32, u32, u32, u32) {
        let dt = self.to_date_time_utc();
        let (_ad, year) = dt.year_ce();
        let month = dt.month0() + 1;
        let day = dt.day0() + 1;
        let hour = dt.hour();
        let minute = dt.minute();
        let second = dt.second();
        (year, month, day, hour, minute, second)
    }
}
