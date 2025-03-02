// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use base64::{engine::general_purpose, Engine as _};

/// Convert u64 to URL safe (e.g. no slashes) base 64 encoding.
pub fn b64_to_u64(s: &String) -> u64 {
    match general_purpose::URL_SAFE.decode(&s) {
        Ok(data) => {
            let bytes: &[u8] = &data;
            match bytes.try_into() {
                Ok(data) => u64::from_le_bytes(data),
                Err(_) => 0u64, // Silently eat the decoding error
            }
        }
        Err(_) => 0u64, // Silently eat the decoding error
    }
}

/// Convert base 64 encoding to u64.
pub fn u64_to_b64(n: u64) -> String {
    let data = n.to_le_bytes();
    general_purpose::URL_SAFE.encode(&data)
}
