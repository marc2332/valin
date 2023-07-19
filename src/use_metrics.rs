use std::borrow::Cow;

use freya::prelude::*;
use skia_safe::scalar;
use skia_safe::textlayout::FontCollection;
use skia_safe::textlayout::ParagraphBuilder;
use skia_safe::textlayout::ParagraphStyle;
use skia_safe::textlayout::TextStyle;
use skia_safe::FontMgr;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::parser::*;
use crate::PanelsManager;

pub fn use_metrics<'a>(
    cx: &'a ScopeState,
    manager: &UseState<PanelsManager>,
    pane_index: usize,
    editor: usize,
    edit_trigger: &UseRef<(UnboundedSender<()>, Option<UnboundedReceiver<()>>)>,
) -> &'a UseState<(SyntaxBlocks, f32)> {
    let metrics = use_state::<(SyntaxBlocks, f32)>(cx, || (Vec::new(), 0.0));

    use_effect(cx, (), move |_| {
        to_owned![metrics, manager];
        let highlight_receiver = &mut edit_trigger.write().1;
        let mut highlight_receiver = highlight_receiver.take().unwrap();

        async move {
            while highlight_receiver.recv().await.is_some() {
                let manager = manager.current();
                let editor = &manager
                    .panel(pane_index)
                    .tab(editor)
                    .as_text_editor()
                    .unwrap();

                let mut font_collection = FontCollection::new();
                font_collection.set_default_font_manager(FontMgr::default(), "Jetbrains Mono");

                let mut style = ParagraphStyle::default();
                let mut text_style = TextStyle::default();
                text_style.set_font_size(manager.font_size());
                style.set_text_style(&text_style);

                let mut paragraph = ParagraphBuilder::new(&style, font_collection);

                let mut longest_line: Vec<Cow<str>> = vec![];

                for line in editor.lines() {
                    let current_longest_width =
                        longest_line.get(0).map(|l| l.len()).unwrap_or_default();

                    let line_len = line.len_chars();

                    if line_len > current_longest_width {
                        longest_line.clear();
                        longest_line.push(line.text)
                    } else if line_len == current_longest_width {
                        longest_line.push(line.text)
                    }
                }

                for line in longest_line {
                    paragraph.add_text(line);
                }

                let mut p = paragraph.build();

                p.layout(scalar::MAX);

                metrics.with_mut(|(syntax_blocks, width)| {
                    parse(editor.rope(), syntax_blocks);
                    *width = p.max_intrinsic_width();
                });
            }
        }
    });

    metrics
}