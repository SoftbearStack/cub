// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use super::MarkdownOptions;

/// Parses string and applies simple cpp rules.
pub(crate) fn cpp(input: &str, options: &MarkdownOptions) -> String {
    let mut line: Vec<char> = Vec::new();
    let mut output: Vec<char> = Vec::new();
    let mut undef = false;

    for ch in input.chars() {
        match ch {
            '\r' => {}
            '\n' => {
                line.push(ch);
                process_line(&mut line, &mut output, &mut undef, options);
            }
            _ => line.push(ch),
        }
    } // for ch
    process_line(&mut line, &mut output, &mut undef, options);

    output.iter().collect()
}

fn process_line(
    line: &mut Vec<char>,
    output: &mut Vec<char>,
    undef: &mut bool,
    options: &MarkdownOptions,
) {
    if !line.is_empty() {
        if line[0] == '#' {
            let text: String = line.iter().collect();
            if text.starts_with("#ifdef ") {
                let var: String = line.drain(7..).collect();
                let var = var.trim();
                *undef = (options.components)(&var, &var).is_none();
            } else if text.starts_with("#ifndef ") {
                let var: String = line.drain(8..).collect();
                let var = var.trim();
                *undef = (options.components)(&var, &var).is_some();
            } else if text.starts_with("#endif") {
                *undef = false;
            } else if !*undef {
                for ch in line.drain(..) {
                    output.push(ch);
                }
            }
        } else if !*undef {
            for ch in line.drain(..) {
                output.push(ch);
            }
        }
        line.clear();
    }
}
