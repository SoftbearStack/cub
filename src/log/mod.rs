// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

/// Thread-safe logging.
mod string_logger;
mod tests;

pub use self::string_logger::StringLogger;
