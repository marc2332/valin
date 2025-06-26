use freya::prelude::*;
use grep::{
    regex::RegexMatcher,
    searcher::{sinks::UTF8, BinaryDetection, SearcherBuilder},
};

use crate::{Overlay, TextArea};

#[component]
pub fn Search() -> Element {
    let mut value = use_signal(String::new);

    let onchange = move |v| {
        if *value.read() != v {
            value.set(v);
        }
    };

    let results = use_memo(move || {
        let mut results = Vec::new();

        if value.read().is_empty() {
            return results;
        }

        let matcher = RegexMatcher::new_line_matcher(&value()).unwrap();

        let mut searcher = SearcherBuilder::new()
            .binary_detection(BinaryDetection::quit(b'\x00'))
            .line_number(true)
            .build();

        let path = "...";

        searcher
            .search_path(
                &matcher,
                path,
                UTF8(|a, b| {
                    print!("{}:{}", a, b);
                    results.push((a, b.to_string()));
                    Ok(true)
                }),
            )
            .unwrap();

        results
    });

    let onsubmit = move |_: String| {
        println!("{value}");
    };

    rsx!(
        Overlay {
             ScrollView {
                height: "400",
                width: "100%",
                for (line, path) in &*results.read() {
                    rect {
                        key: "{line}{path}",
                        label {
                            "{line}:{path}"
                        }
                    }
                }
            }
            TextArea {
                placeholder: "Search...",
                value: "{value}",
                onchange,
                onsubmit,
            }
        }
    )
}
