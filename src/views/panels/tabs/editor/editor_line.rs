use std::rc::Rc;

use dioxus_radio::hooks::use_radio;
use dioxus_sdk::utils::timing::UseDebounce;
use freya::prelude::*;
use lsp_types::Hover;
use skia_safe::textlayout::Paragraph;

use crate::parser::TextNode;
use crate::views::panels::tabs::editor::hover_box::HoverBox;
use crate::views::panels::tabs::editor::AppStateEditorUtils;
use crate::{hooks::UseEdit, utils::create_paragraph};
use crate::{
    lsp::{HoverToText, LspAction, UseLsp},
    state::Channel,
};

use super::SharedRope;

#[derive(Props, Clone)]
pub struct BuilderArgs {
    pub(crate) panel_index: usize,
    pub(crate) tab_index: usize,
    pub(crate) font_size: f32,
    pub(crate) rope: SharedRope,
    pub(crate) line_height: f32,
}

impl PartialEq for BuilderArgs {
    fn eq(&self, other: &Self) -> bool {
        self.panel_index == other.panel_index
            && self.tab_index == other.tab_index
            && self.font_size == other.font_size
            && self.line_height == other.line_height
            && Rc::ptr_eq(&self.rope, &other.rope)
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct EditorLineProps {
    builder_args: BuilderArgs,
    line_index: usize,
    editable: UseEdit,
    lsp: UseLsp,
    hover_location: Signal<Option<(u32, Hover)>>,
    cursor_coords: Signal<CursorPoint>,
    debouncer: UseDebounce<(CursorPoint, u32, Paragraph)>,
}

#[allow(non_snake_case)]
pub fn EditorLine(
    EditorLineProps {
        builder_args:
            BuilderArgs {
                panel_index,
                tab_index,
                font_size,
                rope,
                line_height,
            },
        line_index,
        mut editable,
        lsp,
        hover_location,
        mut cursor_coords,
        mut debouncer,
    }: EditorLineProps,
) -> Element {
    let mut radio_app_state = use_radio(Channel::follow_tab(panel_index, tab_index));

    let onmousedown = move |e: MouseEvent| {
        let mut app_state = radio_app_state.write();
        let editor_tab = app_state.editor_tab_mut(panel_index, tab_index);
        editable.process_event(&EditableEvent::MouseDown(e.data, line_index), editor_tab);
    };

    let onmouseleave = move |_| {
        if lsp.is_supported() {
            lsp.send(LspAction::Clear);
        }
    };

    let onmousemove = {
        to_owned![rope];
        move |e: MouseEvent| {
            let rope = rope.borrow();
            let line_str = rope.line(line_index).to_string();
            let coords = e.get_element_coordinates();
            let data = e.data;

            let mut app_state = radio_app_state.write();
            let editor_tab = app_state.editor_tab_mut(panel_index, tab_index);
            editable.process_event(&EditableEvent::MouseMove(data, line_index), editor_tab);

            if !lsp.is_supported() {
                return;
            }

            cursor_coords.set(coords);

            let paragraph = create_paragraph(&line_str, font_size, radio_app_state);

            if (coords.x as f32) < paragraph.max_intrinsic_width() {
                let coords = cursor_coords.read();
                debouncer.action((*coords, line_index as u32, paragraph));
            } else {
                lsp.send(LspAction::Clear);
            }
        }
    };

    let app_state = radio_app_state.read();
    let editor_tab = app_state.editor_tab(panel_index, tab_index);
    let editor = &editor_tab.editor;
    let longest_width = editor.metrics.longest_width;
    let line = editor.metrics.syntax_blocks.get_line(line_index);
    let highlights = editable.highlights_attr(line_index, editor_tab);
    let gutter_width = font_size * 5.0;

    let is_line_selected = editor.cursor_row() == line_index;

    // Only show the cursor in the active line
    let character_index = if is_line_selected {
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
            background: "{line_background}",
            cross_align: "center",
            if let Some((line, hover)) = hover_location.read().as_ref() {
                if *line == line_index as u32 {
                    if let Some(content) = hover.hover_to_text() {
                        {
                            let cursor_coords = cursor_coords.peek();
                            let offset_x = cursor_coords.x  as f32 + gutter_width;
                            rsx!(
                                rect {
                                    width: "0",
                                    height: "0",
                                    offset_y: "{line_height}",
                                    offset_x: "{offset_x}",
                                    HoverBox {
                                        content
                                    }
                                }
                            )
                        }
                    }
                }
            }
            rect {
                width: "{gutter_width}",
                direction: "horizontal",
                main_align: "end",
                label {
                    margin: "0 20 0 0",
                    font_size: "{font_size}",
                    color: "{gutter_color}",
                    "{line_index + 1} "
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
                cursor_index: "{character_index}",
                cursor_color: "white",
                max_lines: "1",
                cursor_mode: "editable",
                cursor_id: "{line_index}",
                highlights,
                highlight_color: "rgb(65, 65, 65)",
                highlight_mode: "expanded",
                font_size: "{font_size}",
                font_family: "Jetbrains Mono",
                {line.iter().enumerate().map(|(i, (syntax_type, text))| {
                    let text = match text {
                        TextNode::Range(word_pos) => {
                            rope.borrow().slice(word_pos.clone()).to_string()
                        },
                        TextNode::LineOfChars { len, char } => {
                            format!("{char}").repeat(*len)
                        }
                    };

                    rsx!(
                        text {
                            key: "{i}",
                            color: "{syntax_type.color()}",
                            "{text}"
                        }
                    )
                })}
            }
        }
    )
}
