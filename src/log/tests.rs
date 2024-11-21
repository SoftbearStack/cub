// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

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

        let log4 = StringLogger::default();
        log4.trace(format!("not a warning"));
        println!("log4 contains_warnings = {}", log4.contains_warnings());
        log4.warn(format!("this is a warning"));
        println!("log4 contains_warnings = {}", log4.contains_warnings());
        println!("{}", log4.to_string());
    }
}
