use dioxus_radio::hooks::RadioReducer;
use dioxus_radio::prelude::use_radio;
use freya::prelude::*;
use skia_safe::textlayout::{Paragraph, RectHeightStyle, RectWidthStyle};

use crate::hooks::{use_computed, UseDebounce};
use crate::lsp::LspActionData;
use crate::parser::TextNode;
use crate::state::{EditorAction, EditorActionData, TabId};
use crate::views::panels::tabs::editor::hover_box::HoverBox;
use crate::views::panels::tabs::editor::AppStateEditorUtils;
use crate::{hooks::UseEdit, utils::create_paragraph};
use crate::{lsp::LspAction, state::Channel};

use super::SharedRope;

#[derive(Props, Clone)]
pub struct BuilderArgs {
    pub(crate) tab_id: TabId,
    pub(crate) font_size: f32,
    pub(crate) line_height: f32,
}

impl PartialEq for BuilderArgs {
    fn eq(&self, other: &Self) -> bool {
        self.tab_id == other.tab_id
            && self.font_size == other.font_size
            && self.line_height == other.line_height
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct EditorLineProps {
    builder_args: BuilderArgs,
    line_index: usize,
    editable: UseEdit,
    debouncer: UseDebounce<(CursorPoint, u32, Paragraph)>,
    rope: SharedRope,
}

#[allow(non_snake_case)]
pub fn EditorLine(
    EditorLineProps {
        builder_args:
            BuilderArgs {
                tab_id,
                font_size,
                line_height,
            },
        line_index,
        editable,
        mut debouncer,
        rope,
    }: EditorLineProps,
) -> Element {
    let mut radio_app_state = use_radio(Channel::follow_tab(tab_id));

    let app_state = radio_app_state.read();
    let editor_tab = app_state.editor_tab(tab_id);
    let editor = &editor_tab.editor;
    let longest_width = editor.metrics.longest_width;
    let line = editor.metrics.syntax_blocks.get_line(line_index);
    let highlights = editable.highlights_attr(line_index, editor_tab);
    let gutter_width = font_size * 5.0;
    let cursor_reference = editable.cursor_attr();
    let is_line_selected = editor.cursor_row() == line_index;

    let hover_diagnostics = use_computed(&editor.diagnostics, {
        to_owned![rope];
        move |diagnostics| {
            if let Some(diagnostics) = diagnostics.as_ref() {
                if diagnostics.line == line_index as u32 {
                    let rope = rope.borrow();
                    let line_str = rope.line(line_index).to_string();
                    let app_state = radio_app_state.read();
                    let paragraph = create_paragraph(&line_str, font_size, &app_state);
                    let mut text_boxs = paragraph.get_rects_for_range(
                        diagnostics.range.start.character as usize
                            ..diagnostics.range.end.character as usize,
                        RectHeightStyle::default(),
                        RectWidthStyle::default(),
                    );
                    if !text_boxs.is_empty() {
                        return Some((text_boxs.remove(0), diagnostics.content.clone()));
                    }
                }
            }
            None
        }
    });

    let onmousedown = move |e: MouseEvent| {
        radio_app_state.apply(EditorAction {
            tab_id,
            data: EditorActionData::MouseDown {
                data: e.data,
                line_index,
            },
        });
    };

    let onmouseleave = move |_| {
        debouncer.cancel();
        let app_state = radio_app_state.read();
        if let Some(lsp) = app_state.editor_tab_lsp(tab_id) {
            lsp.send(LspAction {
                tab_id,
                action: LspActionData::Clear,
            });
        }
    };

    let onmousemove = {
        to_owned![rope];
        move |e: MouseEvent| {
            let coords = e.get_element_coordinates();

            radio_app_state.apply(EditorAction {
                tab_id,
                data: EditorActionData::MouseMove {
                    data: e.data,
                    line_index,
                },
            });

            let app_state = radio_app_state.read();
            let Some(lsp) = app_state.editor_tab_lsp(tab_id) else {
                return;
            };

            let rope = rope.borrow();
            let line_str = rope.line(line_index).to_string();

            let paragraph = create_paragraph(&line_str, font_size, &app_state);

            if (coords.x as f32) < paragraph.max_intrinsic_width() {
                debouncer.action((coords, line_index as u32, paragraph));
            } else {
                lsp.send(LspAction {
                    tab_id,
                    action: LspActionData::Clear,
                });
            }
        }
    };

    // Only show the cursor in the active line
    let cursor_index = if is_line_selected {
        editor.cursor_col().to_string()
    } else {
        "none".to_string()
    };

    // Only highlight the gutter on the active line
    let gutter_color = if is_line_selected {
        "rgb(235, 235, 235)"
    } else {
        "rgb(135, 135, 135)"
    };

    // Only highlight the active line when there is no text selected
    let line_background = if is_line_selected && !editable.has_any_highlight(editor_tab) {
        "rgb(70, 70, 70)"
    } else {
        "none"
    };

    rsx!(
        rect {
            height: "{line_height}",
            direction: "horizontal",
            background: line_background,
            cross_align: "center",
            rect {
                width: "{gutter_width}",
                direction: "horizontal",
                main_align: "end",
                label {
                    margin: "0 20 0 0",
                    font_size: "{font_size}",
                    color: gutter_color,
                    "{line_index + 1} "
                }
            }
            if let Some((text_box, content)) = hover_diagnostics.borrow().value.as_ref() {
                rect {
                    position: "absolute",
                    position_top: "{line_height}",
                    position_left: "{gutter_width + text_box.rect.left}",
                    HoverBox {
                        content: "{content}"
                    }
                }
            }
            paragraph {
                onmousedown,
                onmousemove,
                onmouseleave,
                min_width: "fill",
                width: "{longest_width}",
                height: "fill",
                main_align: "center",
                cursor_index,
                cursor_color: "white",
                max_lines: "1",
                cursor_reference,
                cursor_mode: "editable",
                cursor_id: "{line_index}",
                highlights,
                highlight_color: "rgb(65, 65, 65)",
                highlight_mode: "expanded",
                font_size: "{font_size}",
                font_family: "Jetbrains Mono",
                {line.iter().enumerate().map(|(i, (syntax_type, text))| {
                    let rope = rope.borrow();
                    let text: Cow<str> = match text {
                        TextNode::Range(word_pos) => {
                            rope.slice(word_pos.clone()).into()
                        },
                        TextNode::LineOfChars { len, char } => {
                            Cow::Owned(char.to_string().repeat(*len))
                        }
                    };

                    rsx!(
                        text {
                            key: "{i}",
                            color: syntax_type.color(),
                            {text}
                        }
                    )
                })}
            }
        }
    )
}
