use std::{ffi::OsStr, path::PathBuf, time::Duration};

use crate::hooks::*;
use crate::lsp::{use_lsp, LspAction};
use crate::state::{EditorView, TabProps};
use crate::tabs::editor::AppStateEditorUtils;
use crate::tabs::editor::BuilderArgs;
use crate::tabs::editor::EditorLine;
use crate::{components::*, state::Channel};

use dioxus_radio::prelude::use_radio;
use dioxus_sdk::utils::timing::use_debounce;
use freya::events::KeyboardEvent;
use freya::prelude::keyboard::Key;
use freya::prelude::keyboard::Modifiers;
use freya::prelude::*;
use lsp_types::Position;

use skia_safe::textlayout::Paragraph;

static LINES_JUMP_ALT: usize = 5;
static LINES_JUMP_CONTROL: usize = 3;

#[allow(non_snake_case)]
pub fn EditorUi(
    TabProps {
        tab_index,
        panel_index,
    }: TabProps,
) -> Element {
    // Subscribe to the changes of this Tab.
    let mut radio_app_state = use_radio(Channel::follow_tab(panel_index, tab_index));

    let app_state = radio_app_state.read();
    let editor_tab = app_state.editor_tab(panel_index, tab_index);
    let editor = &editor_tab.editor;
    let paths = editor.editor_type().paths();

    let mut focus = use_focus_from_id(editor_tab.focus_id);

    // What position in the text the user is hovering
    let hover_location = use_signal(|| None);

    // What location is the user hovering with the mouse
    let cursor_coords = use_signal(CursorPoint::default);

    // Initialize the editable text
    let mut editable = use_edit(&radio_app_state, panel_index, tab_index);

    // The scroll positions of the editor
    let mut scroll_offsets = use_signal(|| (0, 0));

    // Initialize the language server integration
    let lsp = use_lsp(
        &editor.editor_type,
        panel_index,
        tab_index,
        radio_app_state,
        hover_location,
    );

    // Send hover notifications to the LSP only every 300ms and when hovering
    let debouncer = use_debounce(
        Duration::from_millis(300),
        move |(coords, line_index, paragraph): (CursorPoint, u32, Paragraph)| {
            let glyph =
                paragraph.get_glyph_position_at_coordinate((coords.x as i32, coords.y as i32));

            lsp.send(LspAction::Hover(Position::new(
                line_index,
                glyph.position as u32,
            )));
        },
    );

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

    let onglobalclick = move |_: MouseEvent| {
        let is_panel_focused = radio_app_state.read().focused_panel() == panel_index;

        if is_panel_focused {
            editable.process_event(&EditableEvent::Click);
        }
    };

    let onclick = move |e: MouseEvent| {
        e.stop_propagation();
        focus.focus();
        let (is_code_editor_view_focused, is_editor_focused) = {
            let app_state = radio_app_state.read();
            let panel = app_state.panel(panel_index);
            let is_code_editor_view_focused = *app_state.focused_view() == EditorView::Panels;
            let is_editor_focused =
                app_state.focused_panel() == panel_index && panel.active_tab() == Some(tab_index);
            (is_code_editor_view_focused, is_editor_focused)
        };

        if !is_code_editor_view_focused {
            let mut app_state = radio_app_state.write_channel(Channel::Global);
            app_state.set_focused_view(EditorView::Panels);
        }

        if !is_editor_focused {
            let mut app_state = radio_app_state.write_channel(Channel::Global);
            app_state.set_focused_panel(panel_index);
            app_state.panel_mut(panel_index).set_active_tab(tab_index);
        }
    };

    let cursor_reference = editable.cursor_attr();
    let line_height = app_state.line_height();
    let font_size = app_state.font_size();

    let manual_line_height = (font_size * line_height).floor();
    let syntax_blocks_len = editor.metrics.syntax_blocks.len();

    let onkeyup = move |e: KeyboardEvent| {
        let (is_panel_focused, is_editor_focused) = {
            let app_state = radio_app_state.read();
            let panel = app_state.panel(panel_index);
            let is_panel_focused = app_state.focused_panel() == panel_index;
            let is_editor_focused = *app_state.focused_view() == EditorView::Panels
                && panel.active_tab() == Some(tab_index);
            (is_panel_focused, is_editor_focused)
        };

        if is_panel_focused && is_editor_focused {
            editable.process_event(&EditableEvent::KeyUp(e.data));
        }
    };

    let onkeydown = move |e: KeyboardEvent| {
        focus.prevent_navigation();
        e.stop_propagation();
        let (is_panel_focused, is_editor_focused) = {
            let app_state = radio_app_state.read();
            let panel = app_state.panel(panel_index);
            let is_panel_focused = app_state.focused_panel() == panel_index;
            let is_editor_focused = *app_state.focused_view() == EditorView::Panels
                && panel.active_tab() == Some(tab_index);
            (is_panel_focused, is_editor_focused)
        };

        if is_panel_focused && is_editor_focused {
            let current_scroll = scroll_offsets.read().1;
            let lines_jump = (manual_line_height * LINES_JUMP_ALT as f32).ceil() as i32;
            let min_height = -(syntax_blocks_len as f32 * manual_line_height) as i32;
            let max_height = 0; // TODO, this should be the height of the viewport

            let events = match &e.key {
                Key::ArrowUp if e.modifiers.contains(Modifiers::ALT) => {
                    let jump = (current_scroll + lines_jump).clamp(min_height, max_height);
                    scroll_offsets.write().1 = jump;
                    (0..LINES_JUMP_ALT)
                        .map(|_| EditableEvent::KeyDown(e.data.clone()))
                        .collect::<Vec<EditableEvent>>()
                }
                Key::ArrowDown if e.modifiers.contains(Modifiers::ALT) => {
                    let jump = (current_scroll - lines_jump).clamp(min_height, max_height);
                    scroll_offsets.write().1 = jump;
                    (0..LINES_JUMP_ALT)
                        .map(|_| EditableEvent::KeyDown(e.data.clone()))
                        .collect::<Vec<EditableEvent>>()
                }
                Key::ArrowDown | Key::ArrowUp if e.modifiers.contains(Modifiers::CONTROL) => (0
                    ..LINES_JUMP_CONTROL)
                    .map(|_| EditableEvent::KeyDown(e.data.clone()))
                    .collect::<Vec<EditableEvent>>(),
                _ => {
                    vec![EditableEvent::KeyDown(e.data.clone())]
                }
            };

            for event in events {
                editable.process_event(&event);
            }
        }
    };

    rsx!(
        rect {
            width: "100%",
            height: "100%",
            background: "rgb(40, 40, 40)",
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
                onglobalclick,
                onclick,
                cursor_reference,
                EditorScrollView {
                    offset_x: scroll_offsets.read().0,
                    offset_y: scroll_offsets.read().1,
                    onscroll,
                    length: syntax_blocks_len,
                    item_size: manual_line_height,
                    builder_args: BuilderArgs {
                        panel_index,
                        tab_index,
                        font_size,
                        line_height: manual_line_height,
                        rope: editor.rope().clone(),
                    },
                    builder: move |i: usize, builder_args: &BuilderArgs| rsx!(
                        EditorLine {
                            key: "{i}",
                            line_index: i,
                            builder_args: builder_args.clone(),
                            editable,
                            hover_location,
                            debouncer,
                            lsp,
                            cursor_coords,
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
