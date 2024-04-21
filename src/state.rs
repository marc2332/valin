use std::{collections::HashMap, fmt::Display, path::PathBuf};

use dioxus_radio::prelude::{Radio, RadioChannel};
use freya::prelude::Rope;
use tracing::info;

use crate::{
    lsp::{create_lsp, LSPBridge, LspConfig},
    LspStatusSender,
};

pub type RadioAppState = Radio<AppState, Channel>;

pub trait AppStateUtils {
    fn get_focused_data(&self) -> (EditorView, usize, Option<usize>);

    fn editor_mut_data(&self, panel: usize, editor_id: usize) -> Option<(PathBuf, Rope)>;
}

impl AppStateUtils for RadioAppState {
    fn get_focused_data(&self) -> (EditorView, usize, Option<usize>) {
        let app_state = self.read();
        (
            *app_state.focused_view(),
            app_state.focused_panel,
            app_state.panel(app_state.focused_panel).active_tab,
        )
    }

    fn editor_mut_data(&self, panel: usize, editor_id: usize) -> Option<(PathBuf, Rope)> {
        let app_state = self.read();
        let panel: &Panel = app_state.panel(panel);
        let editor = panel.tab(editor_id).as_text_editor();
        editor.map(|editor| (editor.path.clone(), editor.rope.clone()))
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Channel {
    /// Affects global components
    Global,
    /// Affects all tabs
    AllTabs,
    /// Affects individual tab
    Tab {
        panel_index: usize,
        editor_index: usize,
    },
    /// Only affects the active tab
    ActiveTab,
}

impl RadioChannel<AppState> for Channel {
    fn derive_channel(self, app_state: &AppState) -> Vec<Self> {
        match self {
            Self::AllTabs => {
                let mut channels = vec![self, Self::ActiveTab];
                channels.extend(
                    app_state
                        .panels
                        .iter()
                        .enumerate()
                        .flat_map(|(panel_index, panel)| {
                            panel
                                .tabs()
                                .iter()
                                .enumerate()
                                .map(move |(editor_index, _)| Self::Tab {
                                    panel_index,
                                    editor_index,
                                })
                        })
                        .collect::<Vec<Self>>(),
                );

                channels
            }
            Self::Tab {
                panel_index,
                editor_index,
            } => {
                let mut channels = vec![self];
                if app_state.focused_panel == panel_index {
                    let panel = app_state.panel(panel_index);
                    if let Some(active_tab) = panel.active_tab {
                        if active_tab == editor_index {
                            channels.push(Self::ActiveTab);
                        }
                    }
                }
                channels
            }
            _ => vec![self],
        }
    }
}

impl Channel {
    pub fn follow_tab(panel: usize, editor: usize) -> Self {
        Self::Tab {
            panel_index: panel,
            editor_index: editor,
        }
    }
}

use super::EditorData;

#[derive(Clone)]
pub enum PanelTab {
    TextEditor(EditorData),
    Config,
    Welcome,
}

#[derive(PartialEq, Eq)]
pub struct PanelTabData {
    pub edited: bool,
    pub title: String,
    pub id: String,
}

impl PanelTab {
    pub fn get_data(&self) -> PanelTabData {
        match self {
            PanelTab::Config => PanelTabData {
                id: "config".to_string(),
                title: "Config".to_string(),
                edited: false,
            },
            PanelTab::TextEditor(editor) => PanelTabData {
                id: editor.path().to_str().unwrap().to_owned(),
                title: editor
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
                edited: editor.is_edited(),
            },
            PanelTab::Welcome => PanelTabData {
                id: "welcome".to_string(),
                title: "Welcome".to_string(),
                edited: false,
            },
        }
    }

    pub fn as_text_editor(&self) -> Option<&EditorData> {
        if let PanelTab::TextEditor(editor_data) = self {
            Some(editor_data)
        } else {
            None
        }
    }

    pub fn as_text_editor_mut(&mut self) -> Option<&mut EditorData> {
        if let PanelTab::TextEditor(editor_data) = self {
            Some(editor_data)
        } else {
            None
        }
    }
}

#[derive(Clone, Default)]
pub struct Panel {
    pub active_tab: Option<usize>,
    pub tabs: Vec<PanelTab>,
}

impl Panel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active_tab(&self) -> Option<usize> {
        self.active_tab
    }

    pub fn tab(&self, editor: usize) -> &PanelTab {
        &self.tabs[editor]
    }

    pub fn tab_mut(&mut self, editor: usize) -> &mut PanelTab {
        &mut self.tabs[editor]
    }

    pub fn tabs(&self) -> &[PanelTab] {
        &self.tabs
    }

    pub fn set_active_tab(&mut self, active_tab: usize) {
        self.active_tab = Some(active_tab);
    }
}

#[derive(Clone, Default, PartialEq, Copy)]
pub enum EditorView {
    #[default]
    CodeEditor,
    FilesExplorer,
    Commander,
}

impl Display for EditorView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CodeEditor => f.write_str("Code Editor"),
            Self::FilesExplorer => f.write_str("Files Explorer"),
            Self::Commander => f.write_str("Commander"),
        }
    }
}

#[derive(Clone, Default, PartialEq, Copy)]
pub enum EditorSidePanel {
    #[default]
    FileExplorer,
}

#[derive(Clone)]
pub struct AppState {
    pub previous_focused_view: Option<EditorView>,
    pub focused_view: EditorView,
    pub focused_panel: usize,
    pub panels: Vec<Panel>,
    pub font_size: f32,
    pub line_height: f32,
    pub language_servers: HashMap<String, LSPBridge>,
    pub lsp_sender: LspStatusSender,
    pub side_panel: Option<EditorSidePanel>,
}

impl AppState {
    pub fn new(lsp_sender: LspStatusSender) -> Self {
        Self {
            previous_focused_view: None,
            focused_view: EditorView::default(),
            focused_panel: 0,
            panels: vec![Panel::new()],
            font_size: 17.0,
            line_height: 1.2,
            language_servers: HashMap::default(),
            lsp_sender,
            side_panel: Some(EditorSidePanel::default()),
        }
    }

    pub fn toggle_side_panel(&mut self, side_panel: EditorSidePanel) {
        if let Some(current_side_panel) = self.side_panel {
            if current_side_panel == side_panel {
                self.side_panel = None;
                return;
            }
        }

        self.side_panel = Some(side_panel);
    }

    pub fn set_fontsize(&mut self, fontsize: f32) {
        self.font_size = fontsize;
    }

    pub fn set_focused_view(&mut self, focused_view: EditorView) {
        self.previous_focused_view = Some(self.focused_view);

        self.focused_view = focused_view;
    }

    pub fn focused_view(&self) -> &EditorView {
        &self.focused_view
    }

    pub fn set_focused_view_to_previous(&mut self) {
        if let Some(previous_focused_view) = self.previous_focused_view {
            self.focused_view = previous_focused_view;
            self.previous_focused_view = None;
        }
    }

    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    pub fn line_height(&self) -> f32 {
        self.line_height
    }

    pub fn focused_panel(&self) -> usize {
        self.focused_panel
    }

    pub fn push_tab(&mut self, tab: PanelTab, panel: usize, focus: bool) {
        let opened_tab = self.panels[panel]
            .tabs
            .iter()
            .enumerate()
            .find(|(_, t)| t.get_data().id == tab.get_data().id);

        if let Some((tab_index, _)) = opened_tab {
            if focus {
                self.focused_panel = panel;
                self.panels[panel].active_tab = Some(tab_index);
            }
        } else {
            self.panels[panel].tabs.push(tab);

            if focus {
                self.focused_panel = panel;
                self.panels[panel].active_tab = Some(self.panels[panel].tabs.len() - 1);
            }
        }

        info!(
            "Opened tab [panel={panel}] [tab={}]",
            self.panels[panel].tabs.len()
        );
    }

    pub fn close_tab(&mut self, panel: usize, tab: usize) {
        if let Some(active_tab) = self.panels[panel].active_tab {
            let prev_tab = tab > 0;
            let next_tab = self.panels[panel].tabs.get(tab + 1).is_some();
            if active_tab == tab {
                self.panels[panel].active_tab = if next_tab {
                    Some(tab)
                } else if prev_tab {
                    Some(tab - 1)
                } else {
                    None
                };
            } else if active_tab >= tab {
                self.panels[panel].active_tab = Some(active_tab - 1);
            }
        }

        info!(
            "Closed tab [panel={panel}] [tab={}]",
            self.panels[panel].tabs.len()
        );

        let mut panel_tab = self.panels[panel].tabs.remove(tab);

        // Notify the language server that a document was closed
        if let Some(text_editor) = panel_tab.as_text_editor_mut() {
            let language_server_id = text_editor.language_id.language_server();

            // Only if it ever hard LSP support
            if let Some(language_server_id) = language_server_id {
                let language_server = self.language_servers.get_mut(language_server_id);

                // And there was an actual language server running
                if let Some(language_server) = language_server {
                    let file_uri = text_editor.uri();
                    language_server.close_file(file_uri);
                }
            }
        }
    }

    pub fn push_panel(&mut self, panel: Panel) {
        self.panels.push(panel);
    }

    pub fn panels(&self) -> &[Panel] {
        &self.panels
    }

    pub fn panel(&self, panel: usize) -> &Panel {
        &self.panels[panel]
    }

    pub fn panel_mut(&mut self, panel: usize) -> &mut Panel {
        &mut self.panels[panel]
    }

    pub fn set_focused_panel(&mut self, panel: usize) {
        self.focused_panel = panel;
    }

    pub fn close_panel(&mut self, panel: usize) {
        if self.panels.len() > 1 {
            self.panels.remove(panel);
            if self.focused_panel > 0 {
                self.focused_panel -= 1;
            }
        }
    }

    pub fn lsp(&self, lsp_config: &LspConfig) -> Option<&LSPBridge> {
        self.language_servers.get(&lsp_config.language_server)
    }

    pub fn insert_lsp(&mut self, language_server: String, server: LSPBridge) {
        self.language_servers.insert(language_server, server);
    }

    pub async fn get_or_insert_lsp(mut radio: RadioAppState, lsp_config: &LspConfig) -> LSPBridge {
        let server = radio.read().lsp(lsp_config).cloned();
        match server {
            Some(server) => server,
            None => {
                let lsp_sender = radio.read().lsp_sender.clone();
                let server = create_lsp(lsp_config.clone(), lsp_sender).await;
                radio
                    .write_channel(Channel::Global)
                    .insert_lsp(lsp_config.language_server.clone(), server.clone());
                server
            }
        }
    }

    pub fn editor(&self, panel: usize, editor_id: usize) -> &EditorData {
        self.panel(panel).tab(editor_id).as_text_editor().unwrap()
    }

    pub fn editor_mut(&mut self, panel: usize, editor_id: usize) -> &mut EditorData {
        self.panel_mut(panel)
            .tab_mut(editor_id)
            .as_text_editor_mut()
            .unwrap()
    }

    pub fn try_editor_mut(&mut self, panel: usize, editor_id: usize) -> Option<&mut EditorData> {
        self.panel_mut(panel)
            .tab_mut(editor_id)
            .as_text_editor_mut()
    }
}
