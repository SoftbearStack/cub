// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::common::Error;
use std::fmt::Display;
use std::sync::{Arc, Mutex};

/// Thread-safe string logger.
#[derive(Clone, Default)]
pub struct StringLogger {
    pub(crate) debug: bool,
    pub(crate) inner: Arc<Mutex<Vec<String>>>,
}

impl StringLogger {
    /// Call a function which returns a log and either trace that log (if any) or the error.
    pub fn append(&self, line: String, result: Result<String, Error>) -> Result<String, Error> {
        match &result {
            Ok(log) => self.trace(log.to_owned()),
            Err(e) => self.trace(format!("{line} failed\n{e:?}")),
        }
        result
    }

    /// Call a function and trace the result.
    pub fn call<T>(&self, line: String, result: Result<T, Error>) -> Result<T, Error> {
        match &result {
            Ok(_) => self.trace(format!("{line} succeeded")),
            Err(e) => self.trace(format!("{line} failed\n{e:?}")),
        }
        result
    }

    /// Add all lines from the specified logger to this logger.
    pub fn extend(&self, string_logger: &StringLogger) {
        for line in string_logger.inner.lock().unwrap().iter() {
            self.trace(line.clone());
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
            self.inner.lock().unwrap().push(indented_line);
        }
    }

    /// Call a function and trace the result.
    pub fn log<T: Display>(&self, line: String, result: Result<T, Error>) {
        match result {
            Ok(value) => self.trace(format!("{line} succeeded\n{value}")),
            Err(e) => self.trace(format!("{line} failed\n{e:?}")),
        }
    }

    /// Create a new string logger.
    pub fn new(debug: bool) -> Self {
        Self {
            debug,
            inner: Arc::new(Mutex::new(vec![])),
        }
    }

    /// Add a trace line to this logger.
    pub fn trace(&self, line: String) {
        if !line.is_empty() {
            if self.debug {
                println!("{line}");
            }
            self.inner.lock().unwrap().push(line);
        }
    }
}

impl ToString for StringLogger {
    fn to_string(&self) -> String {
        let inner = self.inner.lock().unwrap();
        if inner.is_empty() {
            String::default()
        } else {
            inner.join("\n")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::log::StringLogger;

    #[test]
    fn logger_tests() {
        let log1 = StringLogger::default();

        let bar = 123;
        log1.trace(format!("foo {bar}"));
        log1.trace(format!("bar {bar}"));

        let log2 = StringLogger::default();
        log1.trace(format!("moo {bar}"));
        log1.trace(format!("goo {bar}"));

        let log3 = StringLogger::default();
        log3.extend(&log1);
        log3.extend(&log2);

        println!("{}", log3.to_string());
    }
}
