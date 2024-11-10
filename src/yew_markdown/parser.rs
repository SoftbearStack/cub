// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::MarkdownTag;

const DEBUG: bool = false;

fn emit_anchor(
    line_content: &mut Vec<MarkdownTag>,
    span_content: &mut Vec<char>,
    start_index: usize,
) {
    if start_index != 0 {
        line_content.push(MarkdownTag::Span(
            span_content[..start_index].iter().collect(),
        ));
        *span_content = span_content[start_index..].into();
    }
    if let Some(bracket_index) = span_content.iter().position(|c| *c == ']') {
        let text: String = span_content[1..bracket_index].iter().collect();
        let href_index = bracket_index + 2;
        let href = if href_index < span_content.len() {
            span_content[href_index..span_content.len() - 1]
                .iter()
                .collect()
        } else {
            text.clone()
        };
        line_content.push(MarkdownTag::A(href, text));
        span_content.clear();
        if DEBUG {
            println!("Anchor done, line content is: {line_content:?}");
        }
    }
}

fn emit_markdown(
    output: &mut Vec<MarkdownTag>,
    line_type: LineType,
    line_content: &mut Vec<MarkdownTag>,
) {
    if !line_content.is_empty() {
        match line_type {
            LineType::Bullet(_) => output.push(MarkdownTag::Li(line_content.drain(..).collect())),
            LineType::Heading(n) => {
                output.push(MarkdownTag::H(n, line_content.drain(..).collect()))
            }
            LineType::List(ol) => {
                if ol {
                    output.push(MarkdownTag::Ol(line_content.drain(..).collect()));
                } else {
                    output.push(MarkdownTag::Ul(line_content.drain(..).collect()));
                }
            }
            LineType::Paragraph => output.push(MarkdownTag::P(line_content.drain(..).collect())),
            LineType::Table => output.extend(line_content.drain(..).collect::<Vec<_>>()),
            LineType::None => {}
        }
    }
    if DEBUG {
        println!("Emit {line_type:?}, output is: {output:?}");
    }
}

fn emit_pending(
    output: &mut Vec<MarkdownTag>,
    line_type: LineType,
    line_content: &mut Vec<MarkdownTag>,
    list: &mut Option<bool>,
    bullets: &mut Vec<MarkdownTag>,
) {
    if let Some(ordered) = list {
        if DEBUG {
            println!("End list");
        }
        emit_markdown(bullets, line_type, line_content);
        emit_markdown(output, LineType::List(*ordered), bullets);
        if !line_content.is_empty() {
            if DEBUG {
                println!("WARNING: line content is not empty: {line_content:?}");
            }
            line_content.clear(); // Should be empty anyway.
        }
        *list = None;
    } else {
        emit_markdown(output, line_type, line_content);
    }
}

fn emit_table(
    output: &mut Vec<MarkdownTag>,
    line_content: &mut Vec<MarkdownTag>,
    span_content: &mut Vec<char>,
    titles: Vec<String>,
    body: Vec<Vec<Vec<MarkdownTag>>>,
) {
    if body.is_empty() {
        // An empty table isn't a table.
        if DEBUG {
            println!("Rollback empty table");
        }
        let text = format!("{} {}", titles.join("|"), take_span(span_content, None));
        span_content.extend(text.chars().collect::<Vec<_>>());
    } else {
        if DEBUG {
            println!("Complete table");
        }
        line_content.clear(); // Should be empty anyway.
        line_content.push(MarkdownTag::Table(titles, body));
        emit_markdown(output, LineType::Table, line_content);
    }
}

fn push_span(
    line_content: &mut Vec<MarkdownTag>,
    span_content: &mut Vec<char>,
    end_index: Option<usize>,
) {
    let text = take_span(span_content, end_index);
    // Ignore leading spaces.
    let trimmed_text = text.trim_start().to_string();
    if !(line_content.is_empty() && trimmed_text.is_empty()) {
        line_content.push(MarkdownTag::Span(if line_content.is_empty() {
            trimmed_text
        } else {
            text
        }));
    }
}

fn take_span(span_content: &mut Vec<char>, end_index: Option<usize>) -> String {
    let text: String = span_content
        .drain(..end_index.unwrap_or(span_content.len()))
        .collect();
    if DEBUG {
        println!("Take span '{text}'");
    }
    text
}

/// Parses markdown and returns a list of tokens that maps directly to HTML.
pub(crate) fn tokenize(input: &str) -> Vec<MarkdownTag> {
    let mut bullets: Vec<MarkdownTag> = Vec::new();
    let mut line_content: Vec<MarkdownTag> = Vec::new();
    let mut tokenizer_state = Tokenizer::Start;
    let mut list: Option<bool> = None; // false = <UL>, true = <OL>
    let mut output: Vec<MarkdownTag> = Vec::new();
    let mut span_content: Vec<char> = Vec::new();
    let mut quoted = false;

    if DEBUG {
        println!("DEBUG is on");
    }

    for ch in input.chars() {
        if DEBUG {
            println!(
                "{tokenizer_state:?}: char is '{ch}', span content is '{}'",
                span_content.iter().collect::<String>()
            );
        }
        let matched = if quoted {
            quoted = false;
            false
        } else {
            match ch {
                '\\' => {
                    quoted = true;
                    true
                }
                '\r' => true,
                '\n' => {
                    match tokenizer_state {
                        Tokenizer::Found(line_type) => {
                            // Single newline
                            push_span(&mut line_content, &mut span_content, None);
                            tokenizer_state = Tokenizer::Newline(line_type);
                        }
                        Tokenizer::Newline(line_type) => {
                            // Double newline
                            emit_pending(
                                &mut output,
                                line_type,
                                &mut line_content,
                                &mut list,
                                &mut bullets,
                            );
                        }
                        Tokenizer::PreA(line_type, ']', start_index) => {
                            emit_anchor(&mut line_content, &mut span_content, start_index);
                            span_content.push(ch);
                            tokenizer_state = Tokenizer::Newline(line_type);
                        }
                        Tokenizer::Table(false, titles, body, _) => {
                            emit_table(
                                &mut output,
                                &mut line_content,
                                &mut span_content,
                                titles,
                                body,
                            );
                            tokenizer_state = Tokenizer::Newline(LineType::Paragraph);
                            span_content.push(ch);
                        }
                        Tokenizer::Table(true, titles, mut body, last_row) => {
                            if DEBUG {
                                println!("End table row: {last_row:?}");
                            }
                            body.push(last_row);
                            tokenizer_state = Tokenizer::Table(false, titles, body, vec![]);
                        }
                        Tokenizer::Titles(line_type, titles) => {
                            if DEBUG {
                                println!("Begin table underline, titles are: {titles:?}");
                            }
                            tokenizer_state = Tokenizer::Underline(line_type, false, titles, 0);
                        }
                        Tokenizer::Underline(line_type, true, titles, _count) => {
                            span_content.clear(); // Don't need table underline.
                            emit_pending(
                                &mut output,
                                line_type,
                                &mut line_content,
                                &mut list,
                                &mut bullets,
                            );
                            tokenizer_state = Tokenizer::Table(false, titles, vec![], vec![]);
                        }
                        _ => tokenizer_state = Tokenizer::Newline(LineType::None),
                    };
                    true
                }
                ' ' | '\t' => {
                    match tokenizer_state {
                        Tokenizer::Newline(line_type) => {
                            tokenizer_state = Tokenizer::Indent(line_type)
                        }
                        Tokenizer::PreA(line_type, ']', start_index) => {
                            emit_anchor(&mut line_content, &mut span_content, start_index);
                            span_content.push(ch);
                            tokenizer_state = Tokenizer::Found(line_type);
                        }
                        Tokenizer::PreB(line_type, '2', _) => {
                            // It's not a bold.  For example, "** Hello".
                            span_content.push(ch);
                            tokenizer_state = Tokenizer::Found(line_type);
                        }
                        Tokenizer::PreH(line_type, n) => {
                            emit_pending(
                                &mut output,
                                line_type,
                                &mut line_content,
                                &mut list,
                                &mut bullets,
                            );
                            for _ in 0..n {
                                span_content.pop();
                            }
                            tokenizer_state = Tokenizer::Found(LineType::Heading(n));
                        }
                        Tokenizer::PreLi(line_type, '*' | '-' | '.') => {
                            let n = if let Tokenizer::PreLi(_, n) = tokenizer_state {
                                n
                            } else {
                                '*'
                            };
                            if list.is_some() {
                                if DEBUG {
                                    println!("Continue list");
                                }
                                span_content.clear(); // Ignore bullet.
                                emit_markdown(&mut bullets, line_type, &mut line_content);
                            } else {
                                if DEBUG {
                                    println!("Start list");
                                }
                                span_content.clear(); // Ignore bullet.
                                emit_markdown(&mut output, line_type, &mut line_content);
                                list = Some(n != '*' && n != '-');
                            }
                            tokenizer_state = Tokenizer::Found(LineType::Bullet(n));
                        }
                        Tokenizer::PreLi(_, '1') => {
                            // It's not what it seemed to be.  For example, "1 is not a bullet".
                            span_content.push(ch);
                            tokenizer_state = Tokenizer::Found(LineType::Paragraph);
                        }
                        _ => span_content.push(ch),
                    };
                    true
                }
                '0'..='9' => match tokenizer_state {
                    Tokenizer::Indent(line_type) if ch != '0' => {
                        span_content.push(ch); // In case it's not a bullet.
                        tokenizer_state = Tokenizer::PreLi(line_type, '1');
                        true
                    }
                    Tokenizer::Newline(line_type) if ch != '0' => {
                        span_content.push(' '); // Newline counts as space.
                        span_content.push(ch); // In case it's not a bullet.
                        tokenizer_state = Tokenizer::PreLi(line_type, '1');
                        true
                    }
                    Tokenizer::PreLi(_, '1') => {
                        span_content.push(ch); // In case it's not a bullet.
                        true
                    }
                    Tokenizer::PreH(_, _) | Tokenizer::PreLi(_, '*' | '-') => {
                        // It's not what it seemed to be.  For example, "*1 is not a bullet".
                        span_content.push(ch);
                        tokenizer_state = Tokenizer::Found(LineType::Paragraph);
                        true
                    }
                    Tokenizer::Start => {
                        span_content.push(ch); // In case it's not a heading.
                        tokenizer_state = Tokenizer::PreLi(LineType::Paragraph, '1');
                        true
                    }
                    _ => false,
                },
                '.' => match tokenizer_state {
                    Tokenizer::PreLi(line_type, '1') => {
                        span_content.push(ch); // In case it's not a bullet.
                        tokenizer_state = Tokenizer::PreLi(line_type, '.');
                        true
                    }
                    Tokenizer::PreH(_, _) | Tokenizer::PreLi(_, '*' | '-' | '.') => {
                        // It's not what it seemed to be.  For example, "#. is not a header".
                        span_content.push(ch);
                        tokenizer_state = Tokenizer::Found(LineType::Paragraph);
                        true
                    }
                    _ => false,
                },
                '#' => match tokenizer_state {
                    Tokenizer::Newline(line_type) => {
                        span_content.push(' '); // Newline counts as space.
                        span_content.push(ch); // In case it's not a heading.
                        tokenizer_state = Tokenizer::PreH(line_type, 1);
                        true
                    }
                    Tokenizer::PreH(line_type, n) => {
                        span_content.push(ch); // In case it's not a heading.
                        tokenizer_state = Tokenizer::PreH(line_type, n + 1);
                        true
                    }
                    Tokenizer::Start => {
                        span_content.push(ch); // In case it's not a heading.
                        tokenizer_state = Tokenizer::PreH(LineType::Paragraph, 1);
                        true
                    }
                    _ => false,
                },
                '|' => match tokenizer_state {
                    Tokenizer::Indent(line_type) | Tokenizer::Newline(line_type) => {
                        if DEBUG {
                            println!("Begin table");
                        }
                        tokenizer_state = Tokenizer::Titles(line_type, vec![]);
                        true
                    }
                    Tokenizer::Start => {
                        if DEBUG {
                            println!("Begin table");
                        }
                        tokenizer_state = Tokenizer::Titles(LineType::Paragraph, vec![]);
                        true
                    }
                    Tokenizer::Table(false, titles, body, last_row) => {
                        tokenizer_state = Tokenizer::Table(true, titles, body, last_row);
                        true
                    }
                    Tokenizer::Table(true, titles, body, mut last_row) => {
                        let mut column: Vec<MarkdownTag> = vec![];
                        push_span(&mut line_content, &mut span_content, None);
                        emit_markdown(&mut column, LineType::Paragraph, &mut line_content);
                        last_row.push(column);
                        tokenizer_state = Tokenizer::Table(true, titles, body, last_row);
                        true
                    }
                    Tokenizer::Titles(line_type, mut titles) => {
                        titles.push(take_span(&mut span_content, None));
                        tokenizer_state = Tokenizer::Titles(line_type, titles);
                        true
                    }
                    Tokenizer::Underline(line_type, _, titles, count) => {
                        span_content.push(ch); // In case it's not a table underline.
                        tokenizer_state = Tokenizer::Underline(line_type, true, titles, count + 1);
                        true
                    }
                    _ => false,
                },
                '_' => match tokenizer_state {
                    Tokenizer::Italic(line_type, start_index) => {
                        push_span(&mut line_content, &mut span_content, Some(start_index));
                        let n = span_content.len();
                        if n == 2 {
                            // As an optimization, ignore empty italic, i.e. "__".
                            span_content.clear();
                        } else {
                            // Trim _ from the front and back of span_content.
                            span_content = span_content[1..n].into();
                            line_content.push(MarkdownTag::Em(span_content.drain(..).collect()));
                            if DEBUG {
                                println!("Italic done, line content is: {line_content:?}");
                            }
                        }
                        tokenizer_state = Tokenizer::Found(line_type);
                        true
                    }
                    Tokenizer::Found(line_type) | Tokenizer::Newline(line_type) => {
                        let start_index = span_content.len();
                        if start_index == 0 || span_content[start_index - 1].is_whitespace() {
                            tokenizer_state = Tokenizer::Italic(line_type, start_index);
                        } else {
                            println!("@@@ {start_index} {span_content:?}");
                        }
                        span_content.push(ch);
                        true
                    }
                    Tokenizer::PreA(line_type, ']', start_index) => {
                        emit_anchor(&mut line_content, &mut span_content, start_index);
                        tokenizer_state = Tokenizer::Italic(line_type, span_content.len());
                        span_content.push(ch);
                        true
                    }
                    Tokenizer::Start => {
                        tokenizer_state = Tokenizer::Italic(LineType::Paragraph, 0);
                        span_content.push(ch);
                        true
                    }
                    _ => {
                        println!("@@@#2 {tokenizer_state:?}");
                        false
                    }
                },
                '[' => match tokenizer_state {
                    Tokenizer::Found(line_type) => {
                        tokenizer_state = Tokenizer::PreA(line_type, '[', span_content.len());
                        span_content.push(ch);
                        true
                    }
                    Tokenizer::Newline(line_type) => {
                        emit_pending(
                            &mut output,
                            line_type,
                            &mut line_content,
                            &mut list,
                            &mut bullets,
                        );
                        tokenizer_state =
                            Tokenizer::PreA(LineType::Paragraph, '[', span_content.len());
                        span_content.push(ch);
                        true
                    }
                    Tokenizer::Start => {
                        tokenizer_state = Tokenizer::PreA(LineType::Paragraph, '[', 0);
                        span_content.push(ch);
                        true
                    }
                    _ => false,
                },
                ']' => match tokenizer_state {
                    Tokenizer::PreA(line_type, '[', start_index) => {
                        span_content.push(ch);
                        tokenizer_state = Tokenizer::PreA(line_type, ']', start_index);
                        true
                    }
                    _ => false,
                },
                '(' => match tokenizer_state {
                    Tokenizer::PreA(line_type, ']', start_index) => {
                        span_content.push(ch);
                        tokenizer_state = Tokenizer::PreA(line_type, '(', start_index);
                        true
                    }
                    _ => false,
                },
                ')' => match tokenizer_state {
                    Tokenizer::PreA(line_type, '(', start_index) => {
                        span_content.push(ch);
                        emit_anchor(&mut line_content, &mut span_content, start_index);
                        tokenizer_state = Tokenizer::Found(line_type);
                        true
                    }
                    _ => false,
                },
                '*' | '-' => match tokenizer_state {
                    Tokenizer::Indent(line_type) => {
                        span_content.push(ch); // In case it's not a bullet.
                        tokenizer_state = Tokenizer::PreLi(line_type, ch);
                        true
                    }
                    Tokenizer::Newline(line_type) => {
                        span_content.push(' '); // Newline counts as space.
                        span_content.push(ch); // In case it's not a bullet.
                        tokenizer_state = Tokenizer::PreLi(line_type, ch);
                        true
                    }
                    Tokenizer::PreLi(line_type, '*') => {
                        // It's not what it seemed to be. For example, "**This is not a bullet**".
                        emit_pending(
                            &mut output,
                            line_type,
                            &mut line_content,
                            &mut list,
                            &mut bullets,
                        );
                        tokenizer_state =
                            Tokenizer::PreB(LineType::Paragraph, '2', span_content.len() - 1);
                        span_content.push(ch); // In case it's not a bold.
                        true
                    }
                    Tokenizer::Start => {
                        span_content.push(ch); // In case it's not a heading.
                        tokenizer_state = Tokenizer::PreLi(LineType::Paragraph, ch);
                        true
                    }
                    _ => match ch {
                        '*' => match tokenizer_state {
                            Tokenizer::Bold(line_type, start_index) => {
                                span_content.push(ch); // In case it's not a bold.
                                tokenizer_state = Tokenizer::PostB(line_type, start_index);
                                true
                            }
                            Tokenizer::PostB(line_type, start_index) => {
                                push_span(&mut line_content, &mut span_content, Some(start_index));
                                let n = span_content.len();
                                if n == 3 {
                                    // As an optimization, ignore empty bold, i.e. "***" (final '*' is omitted).
                                    // However, this doesn't actually happen, "****" is omitted verbatim.
                                    span_content.clear();
                                } else {
                                    // Trim ** from the front and back of span_content.
                                    span_content = span_content[2..(n - 1)].into();
                                    line_content
                                        .push(MarkdownTag::B(span_content.drain(..).collect()));
                                    if DEBUG {
                                        println!("Bold done, line content is: {line_content:?}");
                                    }
                                }
                                tokenizer_state = Tokenizer::Found(line_type);
                                true
                            }
                            Tokenizer::PreA(line_type, ']', start_index) => {
                                emit_anchor(&mut line_content, &mut span_content, start_index);
                                tokenizer_state =
                                    Tokenizer::PreB(line_type, '1', span_content.len());
                                span_content.push(ch);
                                true
                            }
                            Tokenizer::PreB(line_type, '1', start_index) => {
                                tokenizer_state = Tokenizer::PreB(line_type, '2', start_index);
                                span_content.push(ch); // In case it's not a bold.
                                true
                            }
                            Tokenizer::PreB(line_type, '2', _) => {
                                // It's not a bold.  For example, "***Hello".
                                span_content.push(ch);
                                tokenizer_state = Tokenizer::Found(line_type);
                                true
                            }
                            Tokenizer::Found(line_type) => {
                                tokenizer_state =
                                    Tokenizer::PreB(line_type, '1', span_content.len());
                                span_content.push(ch); // In case it's not a bold.
                                true
                            }
                            _ => false,
                        },
                        '-' => match tokenizer_state {
                            Tokenizer::Underline(_, true, _, _) => {
                                span_content.push(ch); // In case it's not a table underline.
                                true
                            }
                            _ => false,
                        },
                        _ => false,
                    },
                },
                _ => false,
            }
        };
        if !matched {
            // The default below applies if none of the special cases above matched.
            match tokenizer_state {
                Tokenizer::Bold(_, _)
                | Tokenizer::Found(_)
                | Tokenizer::Italic(_, _)
                | Tokenizer::Table(true, _, _, _)
                | Tokenizer::Titles(_, _)
                | Tokenizer::Underline(_, true, _, _) => {
                    // i.e. Bold, Found (Header, List, Paragraph), or Italic.
                    span_content.push(ch);
                }
                Tokenizer::PreA(line_type, bracket, start_index) => {
                    if bracket == ']' {
                        emit_anchor(&mut line_content, &mut span_content, start_index);
                        tokenizer_state = Tokenizer::Found(line_type);
                    }
                    span_content.push(ch);
                }
                Tokenizer::PreB(line_type, '2', start_index)
                | Tokenizer::PostB(line_type, start_index) => {
                    span_content.push(ch);
                    tokenizer_state = Tokenizer::Bold(line_type, start_index);
                }
                Tokenizer::PreB(line_type, _, _) => {
                    // Just an ordinary span, not a bold.
                    span_content.push(ch);
                    tokenizer_state = Tokenizer::Found(line_type);
                }
                Tokenizer::Newline(LineType::Paragraph) => {
                    span_content.push(' '); // Newline counts as space.
                    span_content.push(ch);
                    tokenizer_state = Tokenizer::Found(LineType::Paragraph);
                }
                Tokenizer::Indent(line_type)
                | Tokenizer::Newline(line_type)
                | Tokenizer::PreH(line_type, _)
                | Tokenizer::PreLi(line_type, _) => {
                    // It's not what it seemed to be. For example, "*a This is not a bullet".
                    emit_pending(
                        &mut output,
                        line_type,
                        &mut line_content,
                        &mut list,
                        &mut bullets,
                    );
                    tokenizer_state = Tokenizer::Found(LineType::Paragraph);
                    span_content.push(ch);
                }
                Tokenizer::Start => {
                    // If no special characters are enountered, the default is an ordinary paragraph.
                    tokenizer_state = Tokenizer::Found(LineType::Paragraph);
                    span_content.push(ch);
                }
                Tokenizer::Table(false, titles, body, _last_row) => {
                    emit_table(
                        &mut output,
                        &mut line_content,
                        &mut span_content,
                        titles,
                        body,
                    );
                    tokenizer_state = Tokenizer::Found(LineType::Paragraph);
                    span_content.push(ch);
                }
                Tokenizer::Underline(line_type, _, titles, _) => {
                    // It wasn't a table underline after all.
                    if DEBUG {
                        println!("Rollback table");
                    }
                    let text = titles.join("|");
                    span_content.extend(text.chars().collect::<Vec<_>>());
                    span_content.push(ch);
                    tokenizer_state = Tokenizer::Found(line_type);
                }
            }
        }
    } // for ch
    push_span(&mut line_content, &mut span_content, None);
    match tokenizer_state {
        Tokenizer::Found(line_type) | Tokenizer::Newline(line_type) => emit_pending(
            &mut output,
            line_type,
            &mut line_content,
            &mut list,
            &mut bullets,
        ),
        _ => {}
    }
    output
}

// Markdown line types.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LineType {
    None,
    Bullet(char),
    Heading(usize),
    List(bool),
    Paragraph,
    Table,
}

/// Tokenizer states
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Tokenizer {
    Start,
    Bold(LineType, usize),
    Found(LineType),
    Indent(LineType),
    Italic(LineType, usize),
    Newline(LineType),
    PostB(LineType, usize),
    PreA(LineType, char, usize),
    PreB(LineType, char, usize),
    PreH(LineType, usize),
    PreLi(LineType, char),
    Table(
        bool,
        Vec<String>,
        Vec<Vec<Vec<MarkdownTag>>>,
        Vec<Vec<MarkdownTag>>,
    ),
    Titles(LineType, Vec<String>),
    Underline(LineType, bool, Vec<String>, usize),
}
