use dioxus_radio::hooks::use_radio;
use dioxus_sdk::utils::timing::UseDebounce;
use freya::prelude::*;
use lsp_types::Hover;
use skia_safe::textlayout::Paragraph;

use crate::tabs::editor::hover_box::HoverBox;
use crate::{hooks::UseEdit, utils::create_paragraph};
use crate::{
    lsp::{HoverToText, LspAction, UseLsp},
    state::Channel,
};

#[derive(Props, Clone, PartialEq)]
pub struct BuilderArgs {
    pub(crate) panel_index: usize,
    pub(crate) editor_index: usize,
    pub(crate) font_size: f32,
    pub(crate) rope: Rope,
    pub(crate) line_height: f32,
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
                editor_index,
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
    let radio_app_state = use_radio(Channel::follow_tab(panel_index, editor_index));

    let onmousedown = move |e: MouseEvent| {
        editable.process_event(&EditableEvent::MouseDown(e.data, line_index));
    };

    let onmouseleave = move |_| {
        if lsp.is_supported() {
            lsp.send(LspAction::Clear);
        }
    };

    let onmouseover = {
        to_owned![rope];
        move |e: MouseEvent| {
            let line_str = rope.line(line_index).to_string();
            let coords = e.get_element_coordinates();
            let data = e.data;

            editable.process_event(&EditableEvent::MouseOver(data, line_index));

            if !lsp.is_supported() {
                return;
            }

            cursor_coords.set(coords);

            let paragraph = create_paragraph(&line_str, font_size);

            if (coords.x as f32) < paragraph.max_intrinsic_width() {
                let coords = cursor_coords.read();
                debouncer.action((*coords, line_index as u32, paragraph));
            } else {
                lsp.send(LspAction::Clear);
            }
        }
    };

    let app_state = radio_app_state.read();
    let editor = app_state.editor(panel_index, editor_index);
    let cursor = editor.cursor();
    let longest_width = editor.metrics.longest_width;
    let line = editor.metrics.syntax_blocks.get_line(line_index);
    let highlights = editable.highlights_attr(line_index);
    let gutter_width = font_size * 3.0;

    let is_line_selected = cursor.row() == line_index;

    // Only show the cursor in the active line
    let character_index = if is_line_selected {
        cursor.col().to_string()
    } else {
        "none".to_string()
    };

    // Only highlight the active line
    let (line_background, gutter_color) = if is_line_selected {
        ("rgb(37, 37, 37)", "rgb(200, 200, 200)")
    } else {
        ("", "rgb(150, 150, 150)")
    };

    rsx!(
        rect {
            height: "{line_height}",
            direction: "horizontal",
            background: "{line_background}",
            {
                if let Some((line, hover)) = hover_location.read().as_ref() {
                    if *line == line_index as u32 {
                        if let Some(content) = hover.hover_to_text() {
                            let cursor_coords = cursor_coords.peek();
                            let offset_x = cursor_coords.x  as f32 + gutter_width;
                            Some(rsx!(
                                rect {
                                    width: "0",
                                    height: "0",
                                    offset_y: "{line_height}",
                                    offset_x: "{offset_x}",
                                    HoverBox {
                                        content: content
                                    }
                                }
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            rect {
                width: "{gutter_width}",
                height: "100%",
                direction: "horizontal",
                label {
                    width: "100%",
                    text_align: "center",
                    font_size: "{font_size}",
                    color: "{gutter_color}",
                    "{line_index + 1} "
                }
            }
            paragraph {
                min_width: "calc(100% - {gutter_width})",
                width: "{longest_width}",
                cursor_index: "{character_index}",
                cursor_color: "white",
                max_lines: "1",
                cursor_mode: "editable",
                cursor_id: "{line_index}",
                onmousedown,
                onmouseover,
                onmouseleave,
                highlights,
                highlight_color: "rgb(65, 65, 65)",
                direction: "horizontal",
                font_size: "{font_size}",
                font_family: "Jetbrains Mono",
                {line.iter().enumerate().map(|(i, (syntax_type, text))| {
                    let text = rope.slice(text.clone());
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
