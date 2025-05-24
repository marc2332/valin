use std::{path::PathBuf, sync::Arc};

use crate::{
    fs::FSReadTransportInterface,
    lsp::{LSPClient, LspAction, LspActionData, LspConfig},
    state::{
        AppSettings, AppState, Channel, EditorCommands, KeyboardShortcuts, PanelTab, PanelTabData,
        RadioAppState, TabId, TabProps,
    },
    views::panels::tabs::editor::TabEditorUtils,
    Args,
};

use freya::prelude::keyboard::Modifiers;
use freya::prelude::*;

use skia_safe::textlayout::FontCollection;
use tracing::info;

use super::{
    commands::{DecreaseFontSizeCommand, IncreaseFontSizeCommand, SaveFileCommand},
    editor_data::{EditorData, EditorType},
    editor_ui::EditorUi,
    SharedRope,
};

/// A tab with an embedded Editor.
pub struct EditorTab {
    pub editor: EditorData,
    pub id: TabId,
    pub focus_id: AccessibilityId,
}

impl PanelTab for EditorTab {
    fn on_close(&mut self, app_state: &mut AppState) {
        // Notify the language server that a document was closed
        let language_id = self.editor.editor_type.language_id();
        let language_server_id = language_id.language_server();

        // Only if it ever hard LSP support
        if let Some(language_server_id) = language_server_id {
            let lsp = app_state.language_servers.get_mut(language_server_id);

            // And there was an actual language server running
            if let Some(lsp) = lsp {
                let file_uri = self.editor.uri();
                if let Some(file_uri) = file_uri {
                    lsp.send(LspAction {
                        tab_id: self.id,
                        action: LspActionData::CloseFile { file_uri },
                    });
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
        let title = self.editor.editor_type.title();
        PanelTabData {
            id: self.id,
            title,
            edited: self.editor.is_edited(),
            focus_id: self.focus_id,
            content_id: self
                .editor
                .editor_type
                .content_id()
                .unwrap_or_else(|| self.id.to_string()),
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
    pub fn new(id: TabId, editor: EditorData) -> Self {
        Self {
            editor,
            id,
            focus_id: UseFocus::new_id(),
        }
    }

    /// Open an EditorTab in the focused panel.
    pub fn open_with(
        mut radio: RadioAppState,
        app_state: &mut AppState,
        path: PathBuf,
        root_path: PathBuf,
        read_transport: Box<dyn FSReadTransportInterface + 'static>,
    ) {
        let rope = SharedRope::default();
        let tab_id = TabId::new();

        let data = EditorData::new(
            EditorType::FS {
                path: path.clone(),
                root_path: root_path.clone(),
            },
            rope.clone(),
            0,
            app_state.clipboard,
            app_state.default_transport.clone(),
        );

        let tab = Self::new(tab_id, data);

        // Dont create the same tab twice
        if !app_state.push_tab(tab, app_state.focused_panel) {
            return;
        }

        // Load file content asynchronously
        spawn_forever({
            to_owned![path];
            async move {
                let content = read_transport.read_to_string(&path).await;
                if let Ok(content) = content {
                    rope.borrow_mut().insert(0, &content);

                    let mut app_state = radio.write_channel(Channel::follow_tab(tab_id));
                    let font_size = app_state.font_size();
                    let font_collection = app_state.font_collection.clone();

                    let tab = app_state.tab_mut(&tab_id);
                    let editor_tab = tab.as_text_editor_mut().unwrap();
                    editor_tab.editor.run_parser();
                    editor_tab
                        .editor
                        .measure_longest_line(font_size, &font_collection);

                    info!("Loaded file content for {path:?}");
                }
            }
        });

        let args = consume_context::<Arc<Args>>();

        let lsp_config = args
            .lsp
            .then(|| {
                LspConfig::new(EditorType::FS {
                    path,
                    root_path: root_path.clone(),
                })
            })
            .flatten();

        if let Some(lsp_config) = lsp_config {
            let (lsp, needs_initialization) = LSPClient::open_with(radio, app_state, &lsp_config);

            // Registry the LSP client
            if needs_initialization {
                app_state.insert_lsp_client(lsp_config.language_server, lsp.clone());
                lsp.send(LspAction {
                    tab_id,
                    action: LspActionData::Initialize(root_path),
                });
            }

            // Open File in LSP Client
            lsp.send(LspAction {
                tab_id,
                action: LspActionData::OpenFile,
            });
        }
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
                    // Pressing `Alt ,`
                    Code::Period if is_pressing_alt => {
                        commands.trigger(IncreaseFontSizeCommand::id());
                    }
                    // Pressing `Alt .`
                    Code::Comma if is_pressing_alt => {
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
