use std::borrow::Cow;
use std::cmp::Ordering;

use freya::prelude::*;
use skia_safe::scalar;
use skia_safe::textlayout::FontCollection;
use skia_safe::textlayout::ParagraphBuilder;
use skia_safe::textlayout::ParagraphStyle;
use skia_safe::textlayout::TextStyle;
use skia_safe::FontMgr;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::hooks::UseManager;
use crate::parser::*;

pub fn use_metrics<'a>(
    cx: &'a ScopeState,
    manager: &UseManager,
    pane_index: usize,
    editor_index: usize,
) -> (&'a UseRef<(SyntaxBlocks, f32)>, &'a UnboundedSender<()>) {
    let metrics = use_ref::<(SyntaxBlocks, f32)>(cx, || (Vec::new(), 0.0));

    let metrics_sender = use_memo(
        cx,
        &(pane_index, editor_index, manager.current().font_size()),
        |_| {
            let (metrics_sender, mut metrics_receiver) = unbounded_channel::<()>();

            metrics_sender.send(()).unwrap();

            to_owned![metrics, manager];
            cx.spawn(async move {
                while metrics_receiver.recv().await.is_some() {
                    let mut font_collection = FontCollection::new();
                    font_collection.set_default_font_manager(FontMgr::default(), "Jetbrains Mono");

                    let mut style = ParagraphStyle::default();
                    let mut text_style = TextStyle::default();
                    text_style.set_font_size(manager.current().font_size());
                    style.set_text_style(&text_style);

                    let mut paragraph_builder = ParagraphBuilder::new(&style, font_collection);

                    let mut longest_line: Vec<Cow<str>> = vec![];

                    let manager = manager.current();

                    let editor = manager
                        .panel(pane_index)
                        .tab(editor_index)
                        .as_text_editor()
                        .unwrap();

                    for line in editor.lines() {
                        let current_longest_width =
                            longest_line.get(0).map(|l| l.len()).unwrap_or_default();

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

                    let (syntax_blocks, width) = &mut *metrics.write();

                    parse(editor.rope(), syntax_blocks);
                    *width = paragraph.longest_line();
                }
            });
            metrics_sender
        },
    );

    (metrics, metrics_sender)
}
