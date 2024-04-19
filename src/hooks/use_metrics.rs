use std::borrow::Cow;
use std::cmp::Ordering;

use freya::prelude::*;
use skia_safe::scalar;
use skia_safe::textlayout::FontCollection;
use skia_safe::textlayout::ParagraphBuilder;
use skia_safe::textlayout::ParagraphStyle;
use skia_safe::textlayout::TextStyle;
use skia_safe::FontMgr;

use crate::{parser::*, state::RadioAppState};

#[derive(Clone, Copy, PartialEq)]
pub struct UseMetrics {
    paragraph_style: Signal<ParagraphStyle>,
    font_collection: Signal<FontCollection>,
    metrics: Signal<(SyntaxBlocks, f32)>,
    radio_app_state: RadioAppState,
    pane_index: usize,
    editor_index: usize,
}

impl UseMetrics {
    pub fn new(
        radio_app_state: RadioAppState,
        metrics: Signal<(SyntaxBlocks, f32)>,
        pane_index: usize,
        editor_index: usize,
    ) -> Self {
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::default(), "Jetbrains Mono");

        let mut paragraph_style = ParagraphStyle::default();
        let mut text_style = TextStyle::default();
        text_style.set_font_size(radio_app_state.read().font_size());
        paragraph_style.set_text_style(&text_style);

        Self {
            paragraph_style: Signal::new(paragraph_style),
            font_collection: Signal::new(font_collection),
            metrics,
            radio_app_state,
            pane_index,
            editor_index,
        }
    }

    pub fn get(&self) -> ReadableRef<Signal<(SyntaxBlocks, f32)>> {
        self.metrics.read()
    }

    pub fn run_metrics(&mut self) {
        let mut paragraph_builder =
            ParagraphBuilder::new(&self.paragraph_style.read(), &*self.font_collection.read());

        let mut longest_line: Vec<Cow<str>> = vec![];

        let app_state = self.radio_app_state.read();

        let editor = app_state
            .panel(self.pane_index)
            .tab(self.editor_index)
            .as_text_editor()
            .unwrap();

        for line in editor.lines() {
            let current_longest_width = longest_line.first().map(|l| l.len()).unwrap_or_default();

            let line_len = line.len_chars();

            match line_len.cmp(&current_longest_width) {
                Ordering::Greater => {
                    longest_line.clear();
                    longest_line.push(line.text)
                }
                Ordering::Equal => longest_line.push(line.text),
                _ => {}
            }
        }

        for line in longest_line {
            paragraph_builder.add_text(line);
        }

        let mut paragraph = paragraph_builder.build();

        paragraph.layout(scalar::MAX);

        let (syntax_blocks, width) = &mut *self.metrics.write();

        parse(editor.rope(), syntax_blocks);
        *width = paragraph.longest_line();
    }
}

pub fn use_metrics(radio: &RadioAppState, pane_index: usize, editor_index: usize) -> UseMetrics {
    let metrics_ref = use_signal::<(SyntaxBlocks, f32)>(|| (SyntaxBlocks::default(), 0.0));

    let mut metrics = use_hook(|| {
        let mut metrics = UseMetrics::new(*radio, metrics_ref, pane_index, editor_index);

        metrics.run_metrics();

        metrics
    });

    metrics.pane_index = pane_index;
    metrics.editor_index = editor_index;

    metrics
}
