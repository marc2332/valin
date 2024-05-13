use std::path::PathBuf;

use crate::state::{
    AppSettings, AppState, EditorCommands, KeyboardShortcuts, PanelTab, PanelTabData,
    RadioAppState, TabProps,
};

use freya::prelude::keyboard::Key;
use freya::prelude::keyboard::Modifiers;
use freya::prelude::*;

use skia_safe::textlayout::FontCollection;

use super::{
    commands::{DecreaseFontSizeCommand, IncreaseFontSizeCommand, SaveFileCommand},
    editor_data::{EditorData, EditorType},
    editor_ui::EditorUi,
};

/// A tab with an embedded Editor.
pub struct EditorTab {
    pub editor: EditorData,
}

impl PanelTab for EditorTab {
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

    fn get_data(&self) -> PanelTabData {
        let (title, id) = self.editor.editor_type.title_and_id();
        PanelTabData {
            id,
            title,
            edited: self.editor.is_edited(),
        }
    }
    fn render(&self) -> fn(TabProps) -> Element {
        EditorUi
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
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

    /// Initialize the EditorTab module.
    pub fn init(
        keyboard_shorcuts: &mut KeyboardShortcuts,
        commands: &mut EditorCommands,
        radio_app_state: RadioAppState,
    ) {
        // Register Commands
        commands.register(IncreaseFontSizeCommand(radio_app_state));
        commands.register(DecreaseFontSizeCommand(radio_app_state));
        commands.register(SaveFileCommand(radio_app_state));

        // Register Shortcuts
        keyboard_shorcuts.register(
            |data: &KeyboardData,
             commands: &mut EditorCommands,
             _radio_app_state: RadioAppState| {
                let is_pressing_alt = data.modifiers == Modifiers::ALT;
                let is_pressing_ctrl = data.modifiers == Modifiers::CONTROL;
                match data.code {
                    // Pressing `Alt +`
                    _ if is_pressing_alt && data.key == Key::Character("+".to_string()) => {
                        commands.trigger(IncreaseFontSizeCommand::id());
                    }
                    // Pressing `Alt -`
                    _ if is_pressing_alt && data.key == Key::Character("-".to_string()) => {
                        commands.trigger(DecreaseFontSizeCommand::id());
                    }
                    // Pressing `Ctrl S`
                    Code::KeyS if is_pressing_ctrl => {
                        commands.trigger(SaveFileCommand::id());
                    }
                    _ => return false,
                }

                true
            },
        )
    }
}
