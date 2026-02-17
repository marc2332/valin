use std::path::PathBuf;

use crate::{
    fs::FSReadTransportInterface,
    state::{
        AppSettings, AppState, Channel, EditorCommands, KeyboardShortcuts, PanelTab, PanelTabData,
        RadioAppState, TabId, TabProps,
    },
    views::panels::tabs::editor::{
        AppStateEditorUtils, EditorData, EditorType, SharedRope, TabEditorUtils,
        commands::{DecreaseFontSizeCommand, IncreaseFontSizeCommand, SaveFileCommand},
        editor_ui::EditorUi,
    },
};

use freya::prelude::*;

use freya::radio::use_radio;
use skia_safe::textlayout::FontCollection;
use tracing::info;

/// A tab with an embedded Editor.
pub struct EditorTab {
    pub editor: EditorData,
    pub id: TabId,
    pub focus_id: AccessibilityId,
}

impl PanelTab for EditorTab {
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
    fn render(&self) -> fn(&TabProps) -> Element {
        |props| {
            let tab_id = props.tab_id;
            let radio_app_state = use_radio(Channel::follow_tab(tab_id));
            let slice = radio_app_state.slice_mut_current(move |s| s.editor_tab_mut(tab_id));
            EditorUi {
                editor: slice.into_writable(),
                font_size: radio_app_state.read().font_size(),
                line_height: radio_app_state.read().line_height(),
            }
            .into()
        }
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
            focus_id: Focus::new_id(),
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
            app_state.default_transport.clone(),
        );

        let tab = Self::new(tab_id, data);

        // Dont create the same tab twice
        if !app_state.push_tab(tab, app_state.focused_panel) {
            return;
        }

        // Load file content asynchronously
        spawn_forever({
            let path = path.clone();
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
            |data: &KeyboardEventData,
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
