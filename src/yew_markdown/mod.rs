// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

mod cpp;
mod parser;
mod renderer;

use self::cpp::cpp;
use self::parser::tokenize;
use self::renderer::yew_html;
use yew::Html;

pub use self::renderer::MarkdownOptions;
pub(crate) use self::renderer::MarkdownTag;

/// Parse markdown `input` and emit Yew `Html`.
pub fn markdown(input: &str, options: &MarkdownOptions) -> Html {
    let preprocessed = cpp(input, options);
    let tokens = tokenize(&preprocessed);
    yew_html(tokens, options)
}

// cargo test --package engine_client --lib -- yew::markdown::tests --nocapture
#[cfg(test)]
mod yew_markdown_tests {
    use crate::yew_markdown::{markdown, MarkdownOptions};
    use yew::{function_component, Html, Properties, ServerRenderer};

    #[tokio::test]
    async fn markdown_tests() {
        let input = r#"
# Hello!
## World!
### Foo!
Paragraph text:
* one,
*  zero **** bold
*  single **I** is bold!
*  just **twobold**,
*   thee wants _three_.
Another paragraph.
 1. 1st is fee
 2.  fie [Your King](king)
 3.   fo [queen]

 4. This fourth item is on a new list
4. As is this fifth item

Yet another paragraph.
 - **eaneybold**
 - meany
 - miney
**moe is a paragraph!**

*# is not a bullet
1 is not a bullet! (same paragraph)
0. is not a bullet (same P)
2 ** 3 is 8 (same paragraph)
2 * 4 = 8 (also, same paragraph)

1. one
2. two
3. three
4. four
5. five
6. six
7. seven
8. eight
9. nine
10. ten
11. eleven

*sterix is not a bullet, either!

[] is not a hyperlink

Yada yada yada. More stuff.
Same paragraph.

Different and last paragraph

"#;

        #[derive(PartialEq, Properties)]
        struct RawHtmlProps {
            html: Html,
        }

        #[function_component(RawHtml)]
        fn raw_html(props: &RawHtmlProps) -> Html {
            props.html.clone()
        }

        let html = ServerRenderer::<RawHtml>::with_props(move || RawHtmlProps {
            html: markdown(input, &MarkdownOptions::default()),
        })
        .render()
        .await;

        let output = format!("{html:?}");

        println!("{output}");
    }
}
