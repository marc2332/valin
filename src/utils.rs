use freya::prelude::*;
use skia_safe::{
    scalar,
    textlayout::{FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle},
    FontMgr,
};

#[allow(non_snake_case)]
pub fn Divider(cx: Scope) -> Element {
    render!(rect {
        background: "rgb(60, 60, 60)",
        height: "100%",
        width: "2",
    })
}

#[allow(non_snake_case)]
pub fn VerticalDivider(cx: Scope) -> Element {
    render!(rect {
        background: "rgb(60, 60, 60)",
        height: "2",
        width: "100%",
    })
}

pub fn create_paragraph(text: &str, font_size: f32) -> Paragraph {
    let mut font_collection = FontCollection::new();
    font_collection.set_default_font_manager(FontMgr::default(), "Jetbrains Mono");

    let mut style = ParagraphStyle::default();
    let mut text_style = TextStyle::default();
    text_style.set_font_size(font_size);
    style.set_text_style(&text_style);

    let mut paragraph_builder = ParagraphBuilder::new(&style, font_collection);

    paragraph_builder.add_text(text);

    let mut paragraph = paragraph_builder.build();

    paragraph.layout(scalar::MAX);

    paragraph
}
