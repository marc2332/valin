use lsp_types::{Hover, HoverContents, MarkedString};

pub trait HoverToText {
    fn hover_to_text(&self) -> Option<String>;
}

impl HoverToText for Hover {
    fn hover_to_text(&self) -> Option<String> {
        let text = match &self.contents {
            HoverContents::Markup(contents) => contents.value.to_owned(),
            HoverContents::Array(contents) => contents
                .iter()
                .map(|v| match v {
                    MarkedString::String(v) => v.to_owned(),
                    MarkedString::LanguageString(text) => text.value.to_owned(),
                })
                .collect::<Vec<String>>()
                .join("\n"),
            HoverContents::Scalar(v) => match v {
                MarkedString::String(v) => v.to_owned(),
                MarkedString::LanguageString(text) => text.value.to_owned(),
            },
        };

        if text == "()" {
            None
        } else {
            Some(text)
        }
    }
}
