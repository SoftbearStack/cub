// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(test)]
mod tests {
    use crate::common::Error;
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

        let log5 = StringLogger::default();
        let ok_result = Ok(log1);
        let _ = log5.append(format!("log5"), ok_result);
        let err_result = Err(Error::String("this is an error".to_string()));
        let _ = log5.append(format!("log5"), err_result);
        println!("{}", log5.to_string());
    }
}
