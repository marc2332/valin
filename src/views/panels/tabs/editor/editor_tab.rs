use std::path::PathBuf;

use crate::{
    fs::{FSReadTransportInterface, FSTransport},
    state::{
        AppSettings, AppState, Channel, EditorCommands, KeyboardShortcuts, PanelTab, PanelTabData,
        RadioAppState, TabId, TabProps,
    },
    views::panels::tabs::editor::{
        AppStateEditorUtils,
        commands::{DecreaseFontSizeCommand, IncreaseFontSizeCommand, SaveFileCommand},
    },
};

use crate::theme::{GITHUB_DARK_EDITOR_THEME, GITHUB_DARK_SYNTAX_THEME};
use freya::code_editor::{CodeEditor, CodeEditorData, LanguageId, Rope};
use freya::prelude::*;
use freya::radio::use_radio;
use tracing::info;

/// A tab with an embedded Editor.
pub struct EditorTab {
    pub(crate) data: CodeEditorData,
    pub(crate) transport: FSTransport,
    pub(crate) id: TabId,
    pub(crate) focus_id: AccessibilityId,
    pub(crate) path: PathBuf,
    pub(crate) icon: freya::prelude::Bytes,
}

impl PanelTab for EditorTab {
    fn on_settings_changed(&mut self, app_settings: &AppSettings) {
        self.data.measure(app_settings.editor.font_size);
    }

    fn get_data(&self) -> PanelTabData {
        PanelTabData {
            id: self.id,
            title: self.content_id(),
            edited: self.data.is_edited(),
            focus_id: self.focus_id,
            content_id: self.content_id(),
            icon: Some(self.icon.clone()),
        }
    }
    fn render(&self) -> fn(&TabProps) -> Element {
        |props| {
            let tab_id = props.tab_id;
            let radio_app_state = use_radio(Channel::follow_tab(tab_id));
            let focus_id = radio_app_state.slice_current(move |s| &s.editor_tab(tab_id).focus_id);
            let editor =
                radio_app_state.slice_mut_current(move |s| &mut s.editor_tab_mut(tab_id).data);
            CodeEditor::new(editor.into_writable(), *focus_id.read())
                .font_size(radio_app_state.read().font_size())
                .line_height(radio_app_state.read().line_height())
                .theme(GITHUB_DARK_EDITOR_THEME)
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
    pub fn new(
        id: TabId,
        data: CodeEditorData,
        transport: FSTransport,
        path: PathBuf,
        icon: freya::prelude::Bytes,
    ) -> Self {
        Self {
            id,
            focus_id: Focus::new_id(),
            data,
            transport,
            path,
            icon,
        }
    }

    pub fn content_id(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_owned()
    }

    /// Open an EditorTab in the focused panel.
    pub fn open_with(
        mut radio: RadioAppState,
        app_state: &mut AppState,
        path: PathBuf,
        read_transport: Box<dyn FSReadTransportInterface + 'static>,
    ) {
        let tab_id = TabId::new();

        let icon = app_state.file_icons.get_file(&path).svg.clone();

        let mut code_data = CodeEditorData::new(
            Rope::new(),
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(LanguageId::parse)
                .unwrap_or(LanguageId::Unknown),
        );
        code_data.set_theme(GITHUB_DARK_SYNTAX_THEME);

        let tab = Self::new(
            tab_id,
            code_data,
            app_state.default_transport.clone(),
            path.clone(),
            icon,
        );

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
                    let mut app_state = radio.write_channel(Channel::follow_tab(tab_id));
                    let font_size = app_state.font_size();

                    let tab = app_state.tab_mut(&tab_id);
                    let editor_tab = tab.as_text_editor_mut().unwrap();
                    editor_tab.data.rope.insert(0, &content);
                    editor_tab.data.parse();
                    editor_tab.data.measure(font_size);

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
