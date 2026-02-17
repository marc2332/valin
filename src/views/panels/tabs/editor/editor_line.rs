use std::borrow::Cow;

use freya::{
    prelude::*,
    text_edit::{EditableEvent, EditorLine, TextEditor},
};

use crate::{syntax::TextNode, views::panels::tabs::editor::EditorData};

#[derive(Clone, PartialEq)]
pub struct EditorLineUI {
    pub(crate) editor: Writable<EditorData>,
    pub(crate) font_size: f32,
    pub(crate) line_height: f32,
    pub(crate) line_index: usize,
}

impl Component for EditorLineUI {
    fn render_key(&self) -> DiffKey {
        DiffKey::from(&self.line_index)
    }
    fn render(&self) -> impl IntoElement {
        let EditorLineUI {
            mut editor,
            font_size,
            line_height,
            line_index,
        } = self.clone();

        let holder = use_state(ParagraphHolder::default);

        let editor_data = editor.read();

        let longest_width = editor_data.metrics.longest_width;
        let line = editor_data.metrics.syntax_blocks.get_line(line_index);
        let highlights = editor_data.get_visible_selection(EditorLine::Paragraph(line_index));
        let gutter_width = font_size * 5.0;
        let is_line_selected = editor_data.cursor_row() == line_index;

        let on_mouse_down = {
            let mut editor = editor.clone();
            move |e: Event<MouseEventData>| {
                editor.write_if(|mut editor_editor| {
                    editor_editor.process(EditableEvent::Down {
                        location: e.element_location,
                        editor_line: EditorLine::Paragraph(line_index),
                        holder: &holder.read(),
                    })
                });
            }
        };

        let on_mouse_up = {
            let mut editor = editor.clone();
            move |_: Event<MouseEventData>| {
                editor.write_if(|mut editor_editor| editor_editor.process(EditableEvent::Release));
            }
        };

        let on_mouse_move = move |e: Event<MouseEventData>| {
            editor.write_if(|mut editor_editor| {
                editor_editor.process(EditableEvent::Move {
                    location: e.element_location,
                    editor_line: EditorLine::Paragraph(line_index),
                    holder: &holder.read(),
                })
            });
        };

        let cursor_index = is_line_selected.then(|| editor_data.cursor_col());
        let gutter_color = if is_line_selected {
            (235, 235, 235)
        } else {
            (135, 135, 135)
        };
        let line_background = if is_line_selected && editor_data.get_selection().is_none() {
            (70, 70, 70).into()
        } else {
            Color::TRANSPARENT
        };

        rect()
            .horizontal()
            .height(Size::px(line_height))
            .background(line_background)
            .font_size(font_size)
            .child(
                rect()
                    .width(Size::px(gutter_width))
                    .height(Size::fill())
                    .padding(Gaps::new(0., 0., 0., 20.))
                    .main_align(Alignment::Center)
                    .child(
                        label()
                            .color(gutter_color)
                            .text(format!("{} ", line_index + 1)),
                    ),
            )
            .child(
                paragraph()
                    .holder(holder.read().clone())
                    .on_mouse_down(on_mouse_down)
                    .on_mouse_up(on_mouse_up)
                    .on_mouse_move(on_mouse_move)
                    .cursor_color(Color::WHITE)
                    .cursor_style(CursorStyle::Block)
                    .cursor_index(cursor_index)
                    .highlights(highlights.map(|h| vec![h]))
                    .width(Size::px(longest_width))
                    .min_width(Size::fill())
                    .height(Size::fill())
                    .font_family("Jetbrains Mono")
                    .max_lines(1)
                    .color((255, 255, 255))
                    .spans_iter(line.iter().map(|span| {
                        let rope = &editor_data.rope;
                        let rope = rope.borrow();
                        let text: Cow<str> = match &span.1 {
                            TextNode::Range(word_pos) => rope.slice(word_pos.clone()).into(),
                            TextNode::LineOfChars { len, char } => {
                                Cow::Owned(char.to_string().repeat(*len))
                            }
                        };
                        Span::new(Cow::Owned(text.to_string())).color(span.0)
                    })),
            )
    }
}
