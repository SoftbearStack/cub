// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use crate::common::Error;
use std::fmt::Display;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct LoggerInner {
    pub(crate) lines: Vec<String>,
    pub(crate) warn: bool,
}

/// Thread-safe string logger.
#[derive(Default)]
pub struct StringLogger {
    pub(crate) debug: bool,
    pub(crate) inner: Arc<Mutex<LoggerInner>>,
}

impl StringLogger {
    /// Append all lines from the specified logger to this logger with the specified indentation.
    pub fn append(&self, string_logger: Self, indentation: Option<&str>) {
        if let (Ok(mut to_inner), Ok(mut from_inner)) =
            (self.inner.lock(), string_logger.inner.lock())
        {
            if !from_inner.lines.is_empty() {
                if from_inner.warn {
                    to_inner.warn = true;
                }
                let mut lines: Vec<_> = from_inner.lines.drain(..).collect();
                if self.debug && !string_logger.debug {
                    println!("{}", lines.join("\n"));
                }
                if let Some(indentation) = indentation {
                    // For efficiency, since re-allocation is necessary anyway, combine the lines.
                    to_inner.lines.push(format!(
                        "{indentation}{}",
                        lines.join("\n").replace('\n', &format!("\n{indentation}"))
                    ));
                } else {
                    to_inner.lines.append(&mut lines);
                }
            }
        }
    }

    /// Call a function and trace the result.
    pub fn call<T>(&self, line: String, result: Result<T, Error>) -> Result<T, Error> {
        match &result {
            Ok(_) => self.trace(format!("{line} succeeded")),
            Err(e) => self.warn(format!("{line} failed\n{e:?}")),
        }
        result
    }

    /// Call a function that returns a string, and then conclude the log with
    /// said result string (if any) as a trace line or else with the error.
    pub fn conclude(
        &self,
        subtask_name: String,
        result: Result<String, Error>,
    ) -> Result<String, Error> {
        match &result {
            Ok(log) => self.trace(format!("{log}\n{subtask_name} succeeded")),
            Err(e) => self.warn(format!("{subtask_name} failed\n{e:?}")),
        }
        result
    }

    /// Whether the log contains any warnings.
    pub fn contains_warnings(&self) -> bool {
        self.inner
            .lock()
            .ok()
            .map(|inner| inner.warn)
            .unwrap_or(false)
    }

    /// Append all lines from the specified logger to this logger without indent.
    pub fn extend(&self, string_logger: Self) {
        self.append(string_logger, None);
    }

    /// Convenience method for simple case.
    pub fn from_string(line: String) -> Self {
        let logger = Self::default();
        logger.trace(line);
        logger
    }

    /// Add an indented trace line to this logger.
    pub fn indent(&self, line: String, indentation: &str) {
        if !line.is_empty() {
            let indented_line = format!(
                "{indentation}{}",
                line.replace('\n', &format!("\n{indentation}"))
            );
            if self.debug {
                println!("{indented_line}");
            }
            if let Ok(mut inner) = self.inner.lock() {
                inner.lines.push(indented_line);
            }
        }
    }

    /// Call a function and trace the result.
    pub fn log<T: Display>(&self, line: String, result: Result<T, Error>) {
        match result {
            Ok(value) => self.trace(format!("{line} succeeded\n{value}")),
            Err(e) => self.warn(format!("{line} failed\n{e:?}")),
        }
    }

    /// Create a new string logger with debug flag.  (To create a new string logger
    /// without specifiying the debug flag, use `StringLogger::default()`.)
    pub fn new(debug: bool) -> Self {
        Self {
            debug,
            inner: Arc::new(Mutex::new(Default::default())),
        }
    }

    /// Prepend all lines from the specified logger to this logger.
    pub fn prepend(&self, string_logger: Self) {
        if let (Ok(mut to_inner), Ok(mut from_inner)) =
            (self.inner.lock(), string_logger.inner.lock())
        {
            if !from_inner.lines.is_empty() {
                if from_inner.warn {
                    to_inner.warn = true;
                }
                to_inner.lines = from_inner
                    .lines
                    .drain(..)
                    .chain(to_inner.lines.drain(..))
                    .collect();
            }
        }
    }

    /// Create a distinct reference to the logger, which is useful for multiple threads.
    pub fn reference(&self) -> Self {
        Self {
            debug: self.debug,
            inner: self.inner.clone(),
        }
    }

    /// Call a function (subtask) that returns a logger and either indent and
    /// trace its log (if any) or else indent the error.
    pub fn subtask(&self, subtask_name: String, result: Result<Self, Error>) -> Result<(), Error> {
        let two_spaces = "  ";
        match result {
            Ok(string_logger) => {
                self.trace(format!("{subtask_name} succeeded:"));
                self.append(string_logger, Some(two_spaces));
                Ok(())
            }
            Err(e) => {
                self.warn(format!("{subtask_name} failed:\n{two_spaces}{e:?}"));
                Err(e)
            }
        }
    }

    /// Add an ordinary trace line to this logger.
    pub fn trace(&self, line: String) {
        if !line.is_empty() {
            if self.debug {
                println!("{line}");
            }
            if let Ok(mut inner) = self.inner.lock() {
                inner.lines.push(line);
            }
        }
    }

    /// Add a warning or error line to this logger.
    pub fn warn(&self, line: String) {
        if !line.is_empty() {
            if self.debug {
                println!("{line}");
            }
            if let Ok(mut inner) = self.inner.lock() {
                inner.lines.push(line);
                inner.warn = true;
            }
        }
    }
}

impl Clone for StringLogger {
    fn clone(&self) -> Self {
        // For efficiency, since re-allocation is necessary anyway, combine the lines.
        Self {
            debug: self.debug,
            inner: Arc::new(Mutex::new(LoggerInner {
                lines: vec![self.to_string()],
                warn: self.contains_warnings(),
            })),
        }
    }
}

impl ToString for StringLogger {
    fn to_string(&self) -> String {
        if let Ok(inner) = self.inner.lock() {
            if inner.lines.is_empty() {
                String::default()
            } else {
                inner.lines.join("\n")
            }
        } else {
            String::default()
        }
    }
}
