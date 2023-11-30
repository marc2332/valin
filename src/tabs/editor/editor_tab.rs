use std::path::PathBuf;
use std::time::Duration;

use crate::components::*;
use crate::hooks::use_manager;
use crate::hooks::EditorView;
use crate::lsp::LanguageId;
use crate::lsp::LspConfig;
use crate::tabs::editor::hooks::use_lsp;

use crate::hooks::*;

use freya::prelude::events::KeyboardEvent;
use freya::prelude::keyboard::Key;
use freya::prelude::keyboard::Modifiers;
use freya::prelude::*;

use lsp_types::Url;

static LINES_JUMP_ALT: usize = 5;
static LINES_JUMP_CONTROL: usize = 3;

#[derive(Props, PartialEq)]
pub struct EditorTabProps {
    pub panel_index: usize,
    pub editor: usize,
    pub language_id: LanguageId,
    pub root_path: PathBuf,
}

#[allow(non_snake_case)]
pub fn EditorTab(cx: Scope<EditorTabProps>) -> Element {
    let lsp_config = LspConfig::new(cx.props.root_path.clone(), cx.props.language_id);
    let manager = use_manager(cx);
    let debouncer = use_debouncer(cx, Duration::from_millis(300));
    let hover_location = use_ref(cx, || None);
    let metrics = use_metrics(cx, &manager, cx.props.panel_index, cx.props.editor);
    let editable = use_edit(cx, &manager, cx.props.panel_index, cx.props.editor, metrics);
    let cursor_coords = use_ref(cx, CursorPoint::default);
    let scroll_offsets = use_ref(cx, || (0, 0));
    let lsp = use_lsp(
        cx,
        cx.props.language_id,
        cx.props.panel_index,
        cx.props.editor,
        &lsp_config,
        &manager,
        hover_location,
    );

    // Focus editor when created
    cx.use_hook(|| {
        let mut manager = manager.write();
        manager.set_focused_panel(cx.props.panel_index);
        manager
            .panel_mut(cx.props.panel_index)
            .set_active_tab(cx.props.editor);
    });

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

    let onglobalclick = {
        to_owned![editable, manager];
        move |_: MouseEvent| {
            let is_panel_focused = manager.current().focused_panel() == cx.props.panel_index;

            if is_panel_focused {
                editable.process_event(&EditableEvent::Click);
            }
        }
    };

    let onclick = {
        to_owned![manager];
        move |_: MouseEvent| {
            let (is_code_editor_view_focused, is_editor_focused) = {
                let manager_ref = manager.current();
                let panel = manager_ref.panel(cx.props.panel_index);
                let is_code_editor_view_focused =
                    *manager_ref.focused_view() == EditorView::CodeEditor;
                let is_editor_focused = manager_ref.focused_panel() == cx.props.panel_index
                    && panel.active_tab() == Some(cx.props.editor);
                (is_code_editor_view_focused, is_editor_focused)
            };

            if !is_code_editor_view_focused {
                let mut manager = manager.global_write();
                manager.set_focused_view(EditorView::CodeEditor);
            }

            if !is_editor_focused {
                let mut manager = manager.global_write();
                manager.set_focused_panel(cx.props.panel_index);
                manager
                    .panel_mut(cx.props.panel_index)
                    .set_active_tab(cx.props.editor);
            }
        }
    };

    let manager_ref = manager.current();
    let cursor_attr = editable.cursor_attr(cx);
    let font_size = manager_ref.font_size();
    let line_height = manager_ref.line_height();
    let manual_line_height = (font_size * line_height).floor();
    let panel = manager_ref.panel(cx.props.panel_index);

    let onkeydown = {
        to_owned![editable, manager];
        move |e: KeyboardEvent| {
            let (is_panel_focused, is_editor_focused) = {
                let manager_ref = manager.current();
                let panel = manager_ref.panel(cx.props.panel_index);
                let is_panel_focused = manager_ref.focused_panel() == cx.props.panel_index;
                let is_editor_focused = *manager_ref.focused_view() == EditorView::CodeEditor
                    && panel.active_tab() == Some(cx.props.editor);
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
        }
    };

    let editor = panel.tab(cx.props.editor).as_text_editor().unwrap();
    let path = editor.path();
    let cursor = editor.cursor();
    let file_uri = Url::from_file_path(path).unwrap();

    render!(
        rect {
            width: "100%",
            height: "100%",
            onkeydown: onkeydown,
            onglobalclick: onglobalclick,
            onclick: onclick,
            cursor_reference: cursor_attr,
            direction: "horizontal",
            background: "rgb(40, 40, 40)",
            padding: "5 0 0 5",
            EditorScrollView {
                offset_x: scroll_offsets.read().0,
                offset_y: scroll_offsets.read().1,
                onscroll: onscroll,
                length: metrics.get().0.len(),
                item_size: manual_line_height,
                options: (cursor, metrics.clone(), editable, lsp.clone(), file_uri, editor.rope().clone(), hover_location.clone(), cursor_coords.clone(), debouncer.clone()),
                font_size: font_size,
                line_height: manual_line_height
            }
        }
    )
}
