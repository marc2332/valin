use std::time::Duration;

use crate::state::EditorView;
use crate::tabs::editor::hooks::use_lsp;
use crate::tabs::editor::BuilderProps;
use crate::tabs::editor::EditorLine;
use crate::{components::*, state::Channel};
use crate::{hooks::*, state::EditorType};

use dioxus_radio::prelude::use_radio;
use freya::events::KeyboardEvent;
use freya::prelude::keyboard::Key;
use freya::prelude::keyboard::Modifiers;
use freya::prelude::*;

use winit::window::CursorIcon;

static LINES_JUMP_ALT: usize = 5;
static LINES_JUMP_CONTROL: usize = 3;

/// Indicates the current focus status of the Editor.
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum EditorStatus {
    /// Default state.
    #[default]
    Idle,
    /// Mouse is hovering the editor.
    Hovering,
}

#[derive(Props, Clone, PartialEq)]
pub struct EditorTabProps {
    pub panel_index: usize,
    pub editor_index: usize,
    pub editor_type: EditorType,
}

#[allow(non_snake_case)]
pub fn EditorTab(props: EditorTabProps) -> Element {
    let mut radio_app_state = use_radio(Channel::follow_tab(props.panel_index, props.editor_index));
    let debouncer = use_debouncer(Duration::from_millis(300));
    let hover_location = use_signal(|| None);
    let metrics = use_metrics(&radio_app_state, props.panel_index, props.editor_index);
    let mut editable = use_edit(
        &radio_app_state,
        props.panel_index,
        props.editor_index,
        &metrics,
    );
    let cursor_coords = use_signal(CursorPoint::default);
    let mut scroll_offsets = use_signal(|| (0, 0));
    let lsp = use_lsp(
        props.editor_type,
        props.panel_index,
        props.editor_index,
        radio_app_state,
        hover_location,
    );
    let platform = use_platform();
    let mut status = use_signal(EditorStatus::default);

    // Focus editor when created
    use_hook(|| {
        {
            let mut app_state = radio_app_state.write();
            app_state.set_focused_panel(props.panel_index);
            app_state
                .panel_mut(props.panel_index)
                .set_active_tab(props.editor_index);
        }
        {
            let mut app_state = radio_app_state.write_channel(Channel::Global);
            app_state.set_focused_view(EditorView::CodeEditor);
        }
    });

    use_drop(move || {
        if *status.read() == EditorStatus::Hovering {
            platform.set_cursor(CursorIcon::default());
        }
    });

    let onmouseenter = move |_| {
        platform.set_cursor(CursorIcon::Text);
        status.set(EditorStatus::Hovering);
    };

    let onmouseleave = move |_| {
        platform.set_cursor(CursorIcon::default());
        status.set(EditorStatus::default());
    };

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
        let is_panel_focused = radio_app_state.read().focused_panel() == props.panel_index;

        if is_panel_focused {
            editable.process_event(&EditableEvent::Click);
        }
    };

    let onclick = move |_: MouseEvent| {
        let (is_code_editor_view_focused, is_editor_focused) = {
            let app_state = radio_app_state.read();
            let panel = app_state.panel(props.panel_index);
            let is_code_editor_view_focused = *app_state.focused_view() == EditorView::CodeEditor;
            let is_editor_focused = app_state.focused_panel() == props.panel_index
                && panel.active_tab() == Some(props.editor_index);
            (is_code_editor_view_focused, is_editor_focused)
        };

        if !is_code_editor_view_focused {
            let mut app_state = radio_app_state.write_channel(Channel::Global);
            app_state.set_focused_view(EditorView::CodeEditor);
        }

        if !is_editor_focused {
            let mut app_state = radio_app_state.write_channel(Channel::Global);
            app_state.set_focused_panel(props.panel_index);
            app_state
                .panel_mut(props.panel_index)
                .set_active_tab(props.editor_index);
        }
    };

    let app_state = radio_app_state.read();
    let cursor_reference = editable.cursor_attr();
    let font_size = app_state.font_size();
    let line_height = app_state.line_height();
    let manual_line_height = (font_size * line_height).floor();
    let panel = app_state.panel(props.panel_index);

    let onkeydown = move |e: KeyboardEvent| {
        let (is_panel_focused, is_editor_focused) = {
            let app_state = radio_app_state.read();
            let panel = app_state.panel(props.panel_index);
            let is_panel_focused = app_state.focused_panel() == props.panel_index;
            let is_editor_focused = *app_state.focused_view() == EditorView::CodeEditor
                && panel.active_tab() == Some(props.editor_index);
            (is_panel_focused, is_editor_focused)
        };

        if is_panel_focused && is_editor_focused {
            let current_scroll = scroll_offsets.read().1;
            let lines_jump = (manual_line_height * LINES_JUMP_ALT as f32).ceil() as i32;
            let min_height = -(metrics.get().0.len() as f32 * manual_line_height) as i32;
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
                    vec![EditableEvent::KeyDown(e.data)]
                }
            };

            for event in events {
                editable.process_event(&event);
            }
        }
    };

    let editor = panel.tab(props.editor_index).as_text_editor().unwrap();
    let cursor = editor.cursor();

    rsx!(
        rect {
            width: "100%",
            height: "100%",
            onmouseenter,
            onmouseleave,
            onkeydown,
            onglobalclick,
            onclick,
            cursor_reference,
            direction: "horizontal",
            background: "rgb(40, 40, 40)",
            padding: "5 0 0 5",
            EditorScrollView {
                offset_x: scroll_offsets.read().0,
                offset_y: scroll_offsets.read().1,
                onscroll,
                length: metrics.get().0.len(),
                item_size: manual_line_height,
                builder_args: (cursor, metrics, editable, lsp, editor.rope().clone(), hover_location, cursor_coords, debouncer, font_size),
                builder: move |i: usize, options: &BuilderProps| rsx!(
                    EditorLine {
                        key: "{i}",
                        line_index: i,
                        options: options.clone(),
                        line_height: manual_line_height,
                    }
                )
            }
        }
    )
}
