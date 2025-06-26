use std::{ffi::OsStr, path::PathBuf, time::Duration};

use crate::hooks::*;
use crate::lsp::{LspAction, LspActionData};
use crate::state::{EditorAction, EditorActionData, TabProps};
use crate::views::panels::tabs::editor::AppStateEditorUtils;
use crate::views::panels::tabs::editor::BuilderArgs;
use crate::views::panels::tabs::editor::EditorLine;
use crate::{components::*, state::Channel};

use dioxus_radio::hooks::RadioReducer;
use dioxus_radio::prelude::use_radio;
use freya::events::KeyboardEvent;
use freya::prelude::*;
use lsp_types::Position;

use skia_safe::textlayout::Paragraph;

#[allow(non_snake_case)]
pub fn EditorUi(TabProps { tab_id }: TabProps) -> Element {
    // Subscribe to the changes of this Tab.
    let mut radio_app_state = use_radio(Channel::follow_tab(tab_id));

    let app_state = radio_app_state.read();
    let editor_tab = app_state.editor_tab(tab_id);
    let editor = &editor_tab.editor;
    let paths = editor.editor_type().paths();
    let rope = editor.rope().clone();

    let mut focus = use_focus_for_id(editor_tab.focus_id);

    // Initialize the editable text
    let editable = use_edit(radio_app_state, tab_id, editor_tab.editor.text_id);

    // The scroll positions of the editor
    let mut scroll_offsets = use_signal(|| (0, 0));

    let mut pressing_shift = use_signal(|| false);
    let mut pressing_alt = use_signal(|| false);

    // Send hover notifications to the LSP only every 300ms and when hovering
    let debouncer = use_debounce(
        Duration::from_millis(300),
        move |(coords, line_index, paragraph): (CursorPoint, u32, Paragraph)| {
            let app_state = radio_app_state.read();
            if let Some(lsp) = app_state.editor_tab_lsp(tab_id) {
                let glyph =
                    paragraph.get_glyph_position_at_coordinate((coords.x as i32, coords.y as i32));
                lsp.send(LspAction {
                    tab_id,
                    action: LspActionData::Hover {
                        position: Position::new(line_index, glyph.position as u32),
                    },
                });
            }
        },
    );

    let line_height = app_state.line_height();
    let font_size = app_state.font_size();

    let line_height = (font_size * line_height).floor();
    let lines_len = editor.metrics.syntax_blocks.len();

    let onscroll = move |(axis, scroll): (Axis, i32)| match axis {
        Axis::X => {
            if scroll_offsets.read().0 != scroll {
                scroll_offsets.write().0 = scroll
            }
        }
        Axis::Y => {
            if scroll_offsets.read().1 != scroll {
                scroll_offsets.write().1 = scroll
            }
        }
    };

    let onclick = move |e: MouseEvent| {
        e.stop_propagation();
        focus.request_focus();
        radio_app_state.apply(EditorAction {
            tab_id,
            data: EditorActionData::Click,
        });
    };

    let onkeyup = move |e: KeyboardEvent| {
        match &e.key {
            Key::Shift => {
                pressing_shift.set(false);
            }
            Key::Alt => {
                pressing_alt.set(false);
            }
            _ => {}
        };

        radio_app_state.apply(EditorAction {
            tab_id,
            data: EditorActionData::KeyUp { data: e.data },
        });
    };

    let onkeydown = move |e: KeyboardEvent| {
        e.stop_propagation();

        match &e.key {
            Key::Shift => {
                pressing_shift.set(true);
            }
            Key::Alt => {
                pressing_alt.set(true);
            }
            _ => {}
        };

        let channel = radio_app_state.apply(EditorAction {
            tab_id,
            data: EditorActionData::KeyDown {
                data: e.data.clone(),
                scroll_offsets,
                line_height,
                lines_len,
            },
        });

        // If a change was applied the default behavior will be prevented
        if channel.is_current() && channel.is_select().is_some() {
            e.prevent_default();
        }
    };

    rsx!(
        rect {
            width: "100%",
            height: "100%",
            background: "rgb(29, 32, 33)",
            if let Some((path, root_path)) = paths {
                FilePath {
                    path: path.clone(),
                    root_path: root_path.clone(),
                }
            }
            rect {
                a11y_id: focus.attribute(),
                onkeydown,
                onkeyup,
                onclick,
                EditorScrollView {
                    offset_x: scroll_offsets.read().0,
                    offset_y: scroll_offsets.read().1,
                    onscroll,
                    length: lines_len,
                    item_size: line_height,
                    builder_args: BuilderArgs {
                        tab_id,
                        font_size,
                        line_height,
                    },
                    pressing_alt,
                    pressing_shift,
                    builder: move |i: usize, builder_args: &BuilderArgs| rsx!(
                        EditorLine {
                            key: "{i}",
                            line_index: i,
                            builder_args: builder_args.clone(),
                            editable,
                            debouncer,
                            rope: rope.clone()
                        }
                    )
                }
            }
        }
    )
}

#[allow(non_snake_case)]
#[component]
fn FilePath(path: PathBuf, root_path: PathBuf) -> Element {
    let relative_path = if path == root_path {
        path
    } else {
        path.strip_prefix(&root_path).unwrap().to_path_buf()
    };

    let mut components = relative_path.components().enumerate().peekable();

    let mut children = Vec::new();

    while let Some((i, component)) = components.next() {
        let is_last = components.peek().is_none();
        let text: &OsStr = component.as_ref();

        children.push(rsx!(
            rect {
                key: "{i}",
                direction: "horizontal",
                label {
                    "{text.to_str().unwrap()}"
                }
                if !is_last {
                    label {
                        margin: "0 6",
                        ">"
                    }
                }
            }
        ))
    }

    rsx!(
        rect {
            width: "100%",
            direction: "horizontal",
            color: "rgb(215, 215, 215)",
            padding: "0 10",
            height: "28",
            cross_align: "center",
            {children.into_iter()}
        }
    )
}
