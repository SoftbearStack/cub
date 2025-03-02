// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

#[cfg(test)]
mod tests {
    use crate::common::Error;
    use crate::log::StringLogger;

    #[test]
    fn logger_tests() {
        println!("Testing logger");
        let log1 = StringLogger::default();

        let bar = 123;
        log1.trace(format!("foo {bar}"));
        log1.trace(format!("bar {bar}"));
        println!("Testing trace:\n{}", log1.to_string());

        let log2 = StringLogger::from_string(format!("moo {bar}"));
        println!("Testing from_string:\n{}", log2.to_string());
        let log2b = log2.reference();
        log2b.trace(format!("reference worked"));
        println!("Testing reference:\n{}", log2b.to_string());
        println!(
            "Testing that other reference is the same:\n{}",
            log2.to_string()
        );

        let log3 = StringLogger::default();
        log3.extend(log1.clone());
        log3.extend(log2);
        println!("Testing extend:\n{}", log3.to_string());
        println!("OK result should be:\n{}", log1.to_string());

        let log4 = StringLogger::default();
        log4.trace(format!("not a warning"));
        println!(
            "Testing warnings false:\nlog4 contains_warnings = {}",
            log4.contains_warnings()
        );
        log4.warn(format!("this is a warning"));
        println!(
            "Testing warnings true:\nlog4 contains_warnings = {}",
            log4.contains_warnings()
        );
        println!("Testing warnings text:\n{}", log4.to_string());

        let log5 = StringLogger::default();
        let ok_result = Ok(log1);
        let _ = log5.subtask(format!("log5"), ok_result);
        let err_result = Err(Error::String("this is an error".to_string()));
        let _ = log5.subtask(format!("log5"), err_result);
        println!("Testing subtask:\n{}", log5.to_string());

        let log6 = StringLogger::default();
        log6.trace(format!("This comes after"));
        log6.prepend(log5);
        println!("Testing prepend:\n{}", log6.to_string());
    }
}
