use std::borrow::Cow;
use std::cmp::Ordering;

use freya::prelude::*;
use skia_safe::scalar;
use skia_safe::textlayout::FontCollection;
use skia_safe::textlayout::ParagraphBuilder;
use skia_safe::textlayout::ParagraphStyle;
use skia_safe::textlayout::TextStyle;
use skia_safe::FontMgr;

use crate::parser::*;

pub struct EditorMetrics {
    pub(crate) font_collection: FontCollection,
    pub(crate) syntax_blocks: SyntaxBlocks,
    pub(crate) longest_width: f32,
}

impl EditorMetrics {
    pub fn new() -> Self {
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::default(), "Jetbrains Mono");

        Self {
            font_collection,
            syntax_blocks: SyntaxBlocks::default(),
            longest_width: 0.0,
        }
    }

    pub fn measure_longest_line(&mut self, font_size: f32, rope: &Rope) {
        let mut paragraph_style = ParagraphStyle::default();
        let mut text_style = TextStyle::default();
        text_style.set_font_size(font_size);
        paragraph_style.set_text_style(&text_style);
        let mut paragraph_builder = ParagraphBuilder::new(&paragraph_style, &self.font_collection);

        let mut longest_line: Vec<Cow<str>> = vec![];

        for line in rope.lines() {
            let current_longest_width = longest_line.first().map(|l| l.len()).unwrap_or_default();

            let line_len = line.len_chars();

            match line_len.cmp(&current_longest_width) {
                Ordering::Greater => {
                    longest_line.clear();
                    longest_line.push(line.into())
                }
                Ordering::Equal => longest_line.push(line.into()),
                _ => {}
            }
        }

        for line in longest_line {
            paragraph_builder.add_text(line);
        }

        let mut paragraph = paragraph_builder.build();

        paragraph.layout(scalar::MAX);

        self.longest_width = paragraph.longest_line();
    }

    pub fn run_parser(&mut self, rope: &Rope) {
        parse(rope, &mut self.syntax_blocks);
    }
}
