// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

/// Used to avoid serializing booleans with value `false`.
/// # Example
/// `#[serde(default, skip_serializing_if = "is_default")]`
pub fn is_default<T: Default + PartialEq>(x: &T) -> bool {
    x == &T::default()
}
