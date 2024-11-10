// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

/// Canonicalize name.
mod canonicalize;
/// IDs of various bit sizes.
mod id;
mod tests;
/// Thin wrappers around Unix timestamp (non leap milliseconds since 1970).
mod time;

pub use self::canonicalize::{canonicalize, CanonicalizationError};
pub use self::id::{ID32, ID64};
pub use self::time::{NonZeroUnixMillis, NonZeroUnixSeconds, UnixMillis, UnixTime};
