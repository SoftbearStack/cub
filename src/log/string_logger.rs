// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::common::Error;
use std::fmt::Display;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct LoggerInner {
    pub(crate) lines: Vec<String>,
    pub(crate) warn: bool,
}

/// Thread-safe string logger.
#[derive(Clone, Default)]
pub struct StringLogger {
    pub(crate) debug: bool,
    pub(crate) inner: Arc<Mutex<LoggerInner>>,
}

impl StringLogger {
    /// Call a function which returns a logger and either indent and trace its log
    /// (if any) or else the error.
    pub fn append(
        &self,
        line: String,
        result: Result<StringLogger, Error>,
    ) -> Result<StringLogger, Error> {
        let two_spaces = "  ";
        match &result {
            Ok(string_logger) => {
                self.trace(format!("{line} succeeded:"));
                if let (Ok(mut to_inner), Ok(from_inner)) =
                    (self.inner.lock(), string_logger.inner.lock())
                {
                    if !from_inner.lines.is_empty() {
                        if from_inner.warn {
                            to_inner.warn = true;
                        }
                        let lines =
                            format!("{two_spaces}{}", from_inner.lines.join(&format!("\n{two_spaces}")));
                        if self.debug {
                            println!("{lines}");
                        }
                        to_inner.lines.push(lines);
                    }
                }
            }
            Err(e) => self.warn(format!("{line} failed:\n{two_spaces}{e:?}")),
        }
        result
    }

    /// Call a function and trace the result.
    pub fn call<T>(&self, line: String, result: Result<T, Error>) -> Result<T, Error> {
        match &result {
            Ok(_) => self.trace(format!("{line} succeeded")),
            Err(e) => self.warn(format!("{line} failed\n{e:?}")),
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

    /// Add all lines from the specified logger to this logger.
    pub fn extend(&self, string_logger: &StringLogger) {
        if let (Ok(mut to_inner), Ok(from_inner)) = (self.inner.lock(), string_logger.inner.lock())
        {
            if !from_inner.lines.is_empty() {
                if from_inner.warn {
                    to_inner.warn = true;
                }
                let lines = from_inner.lines.join("\n");
                if self.debug {
                    println!("{lines}");
                }
                to_inner.lines.push(lines);
            }
        }
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

    /// Create a new string logger.
    pub fn new(debug: bool) -> Self {
        Self {
            debug,
            inner: Arc::new(Mutex::new(Default::default())),
        }
    }

    /// Call a function which returns a string and either prepend that string
    /// (if any) as trace line(s) or else the error.
    pub fn prepend(&self, line: String, result: Result<String, Error>) -> Result<String, Error> {
        match &result {
            Ok(log) => self.trace(format!("{log}\n{line} succeeded")),
            Err(e) => self.warn(format!("{line} failed\n{e:?}")),
        }
        result
    }

    /// Add a trace line to this logger.
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
