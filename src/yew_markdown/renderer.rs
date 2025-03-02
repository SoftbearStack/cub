// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use yew::{html, Html};

/// Markdown renderer options.
pub struct MarkdownOptions {
    /// fn(href, content) -> Html
    #[allow(clippy::type_complexity)]
    pub components: Box<dyn Fn(&str, &str) -> Option<Html>>,
    /// Start headings with specified level instead of `<h1>`.
    pub h_level: usize,
}

impl Default for MarkdownOptions {
    fn default() -> Self {
        Self {
            components: Box::new(|_, _| None),
            h_level: 3,
        }
    }
}

/// HTML tags that are created from markdown.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MarkdownTag {
    A(String, String),
    B(String),
    Em(String),
    H(usize, Vec<MarkdownTag>),
    Li(Vec<MarkdownTag>),
    Ol(Vec<MarkdownTag>),
    P(Vec<MarkdownTag>),
    Span(String),
    Table(Vec<String>, Vec<Vec<Vec<MarkdownTag>>>),
    Ul(Vec<MarkdownTag>),
}

/// Creates Yew object hierarchy by recursively walking markdown tokens.
pub(crate) fn yew_html(tokens: Vec<MarkdownTag>, options: &MarkdownOptions) -> Html {
    tokens
        .into_iter()
        .map(|t| match t {
            MarkdownTag::A(href, content) => {
                (options.components)(&href, &content).unwrap_or_else(|| {
                    html! {
                        <a {href}>{content}</a>
                    }
                })
            }
            MarkdownTag::B(text) => html! {
                <b>{text}</b>
            },
            MarkdownTag::Em(text) => html! {
                <em>{text}</em>
            },
            MarkdownTag::H(n, content) => {
                let k = options.h_level + n - 1;
                match k {
                    1 => html! {<h1>{yew_html(content, options)}</h1>},
                    2 => html! {<h2>{yew_html(content, options)}</h2>},
                    3 => html! {<h3>{yew_html(content, options)}</h3>},
                    4 => html! {<h4>{yew_html(content, options)}</h4>},
                    5 => html! {<h5>{yew_html(content, options)}</h5>},
                    _ => html! {<h6>{yew_html(content, options)}</h6>},
                }
            }
            MarkdownTag::Li(content) => html! {
                <li>{yew_html(content, options)}</li>
            },
            MarkdownTag::Ol(content) => html! {
                <ol>{yew_html(content, options)}</ol>
            },
            MarkdownTag::P(content) => html! {
                <p>{yew_html(content, options)}</p>
            },
            MarkdownTag::Span(text) => html! {
                {text}
            },
            MarkdownTag::Table(titles, body) => html! {
                <table>
                    <thead>
                        <tr>
                            { titles.iter().map(|t| html!{ <th>{t}</th> }).collect::<Html>() }
                        </tr>
                    </thead>
                    <tbody>
                        {
                            body.into_iter().map(|row| html! {
                                <tr>
                                {
                                    row.into_iter().map(|col| html! {
                                        <td>
                                            {
                                                if col.len() == 1 {
                                                    match &col[0] {
                                                        // Eliminate <p> if there is only one <p> inside <td>
                                                        MarkdownTag::P(tags) => yew_html(tags.to_vec(), options),
                                                        _ => yew_html(col, options),
                                                    }
                                                } else {
                                                    yew_html(col, options)
                                                }
                                            }
                                        </td>
                                    }).collect::<Html>()
                                }
                                </tr>
                            }).collect::<Html>()
                        }
                    </tbody>
                </table>
            },
            MarkdownTag::Ul(content) => html! {
                <ul>{yew_html(content, options)}</ul>
            },
        })
        .collect::<Html>()
}
