// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

/// Thread-safe logging.
mod string_logger;
mod tests;

pub use self::string_logger::StringLogger;
