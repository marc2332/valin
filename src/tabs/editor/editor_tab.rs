use std::{path::PathBuf, time::Duration};

use crate::state::{EditorView, PanelTab, TabProps};
use crate::tabs::editor::AppStateEditorUtils;
use crate::tabs::editor::BuilderArgs;
use crate::tabs::editor::EditorLine;
use crate::{components::*, state::Channel};
use crate::{hooks::*, state::AppSettings};
use crate::{
    lsp::{use_lsp, LspAction},
    state::{AppState, PanelTabData},
};

use dioxus_radio::prelude::use_radio;
use dioxus_sdk::utils::timing::use_debounce;
use freya::events::KeyboardEvent;
use freya::prelude::keyboard::Key;
use freya::prelude::keyboard::Modifiers;
use freya::prelude::*;
use lsp_types::Position;

use skia_safe::textlayout::{FontCollection, Paragraph};
use winit::window::CursorIcon;

use super::editor_data::{EditorData, EditorType};

static LINES_JUMP_ALT: usize = 5;
static LINES_JUMP_CONTROL: usize = 3;

/// A tab with an embedded Editor.
pub struct EditorTab {
    pub editor: EditorData,
}

impl PanelTab for EditorTab {
    fn get_data(&self) -> PanelTabData {
        let (title, id) = self.editor.editor_type.title_and_id();
        PanelTabData {
            id,
            title,
            edited: self.editor.is_edited(),
        }
    }
    fn render(&self) -> fn(TabProps) -> Element {
        render
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn on_close(&mut self, app_state: &mut AppState) {
        // Notify the language server that a document was closed
        let language_id = self.editor.editor_type.language_id();
        let language_server_id = language_id.language_server();

        // Only if it ever hard LSP support
        if let Some(language_server_id) = language_server_id {
            let language_server = app_state.language_servers.get_mut(language_server_id);

            // And there was an actual language server running
            if let Some(language_server) = language_server {
                let file_uri = self.editor.uri();
                if let Some(file_uri) = file_uri {
                    language_server.close_file(file_uri);
                }
            }
        }
    }

    fn on_settings_changed(
        &mut self,
        app_settings: &AppSettings,
        font_collection: &FontCollection,
    ) {
        self.editor
            .measure_longest_line(app_settings.editor.font_size, font_collection);
    }
}

impl EditorTab {
    /// Open an EditorTab in the focused panel.
    pub fn open_with(app_state: &mut AppState, path: PathBuf, root_path: PathBuf, content: String) {
        let data = EditorData::new(
            EditorType::FS { path, root_path },
            Rope::from(content),
            (0, 0),
            app_state.clipboard,
            app_state.default_transport.clone(),
            app_state.settings.editor.font_size,
            &app_state.font_collection.clone(),
        );

        app_state.push_tab(Self { editor: data }, app_state.focused_panel, true);
    }
}

/// Indicates the current focus status of the Editor.
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum EditorStatus {
    /// Default state.
    #[default]
    Idle,
    /// Mouse is hovering the editor.
    Hovering,
}

#[allow(non_snake_case)]
fn render(
    TabProps {
        tab_index,
        panel_index,
    }: TabProps,
) -> Element {
    // Subscribe to the changes of this Tab.
    let mut radio_app_state = use_radio(Channel::follow_tab(panel_index, tab_index));

    // Automatically focus this editor when created
    use_hook(|| {
        {
            let mut app_state = radio_app_state.write();
            app_state.set_focused_panel(panel_index);
            app_state.panel_mut(panel_index).set_active_tab(tab_index);
        }
        {
            let mut app_state = radio_app_state.write_channel(Channel::Global);
            app_state.set_focused_view(EditorView::CodeEditor);
        }
    });

    let app_state = radio_app_state.read();
    let editor_tab = app_state.editor_tab(panel_index, tab_index);
    let editor = &editor_tab.editor;

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
    let platform = use_platform();
    let mut status = use_signal(EditorStatus::default);

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
        let is_panel_focused = radio_app_state.read().focused_panel() == panel_index;

        if is_panel_focused {
            editable.process_event(&EditableEvent::Click);
        }
    };

    let onclick = move |_: MouseEvent| {
        let (is_code_editor_view_focused, is_editor_focused) = {
            let app_state = radio_app_state.read();
            let panel = app_state.panel(panel_index);
            let is_code_editor_view_focused = *app_state.focused_view() == EditorView::CodeEditor;
            let is_editor_focused =
                app_state.focused_panel() == panel_index && panel.active_tab() == Some(tab_index);
            (is_code_editor_view_focused, is_editor_focused)
        };

        if !is_code_editor_view_focused {
            let mut app_state = radio_app_state.write_channel(Channel::Global);
            app_state.set_focused_view(EditorView::CodeEditor);
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

    let onkeydown = move |e: KeyboardEvent| {
        let (is_panel_focused, is_editor_focused) = {
            let app_state = radio_app_state.read();
            let panel = app_state.panel(panel_index);
            let is_panel_focused = app_state.focused_panel() == panel_index;
            let is_editor_focused = *app_state.focused_view() == EditorView::CodeEditor
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
                    vec![EditableEvent::KeyDown(e.data)]
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
    )
}
