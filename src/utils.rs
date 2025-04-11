use freya::prelude::*;
use skia_safe::{
    scalar,
    textlayout::{Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle},
};

use crate::state::AppState;

#[allow(non_snake_case)]
pub fn Divider() -> Element {
    rsx!(rect {
        background: "rgb(56, 59, 66)",
        height: "100%",
        width: "1",
    })
}

#[allow(non_snake_case)]
pub fn VerticalDivider() -> Element {
    rsx!(rect {
        background: "rgb(56, 59, 66)",
        height: "1",
        width: "100%",
    })
}

pub fn create_paragraph(text: &str, font_size: f32, app_state: &AppState) -> Paragraph {
    let mut style = ParagraphStyle::default();
    let mut text_style = TextStyle::default();
    text_style.set_font_size(font_size);
    style.set_text_style(&text_style);

    let mut paragraph_builder = ParagraphBuilder::new(&style, &app_state.font_collection);

    paragraph_builder.add_text(text);

    let mut paragraph = paragraph_builder.build();

    paragraph.layout(scalar::MAX);

    paragraph
}
