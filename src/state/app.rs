use std::{collections::HashMap, path::PathBuf};

use dioxus_radio::prelude::{Radio, RadioChannel};
use dioxus_sdk::clipboard::UseClipboard;
use freya::prelude::Rope;
use skia_safe::{textlayout::FontCollection, FontMgr};
use tracing::info;

use crate::{
    fs::FSTransport,
    lsp::{create_lsp_client, LSPClient, LspConfig},
    LspStatusSender, TreeItem,
};

use super::{EditorData, EditorType, EditorView, Panel, PanelTab};

pub type RadioAppState = Radio<AppState, Channel>;

pub trait AppStateUtils {
    fn get_focused_data(&self) -> (EditorView, usize, Option<usize>);

    fn editor_mut_data(
        &self,
        panel: usize,
        editor_id: usize,
    ) -> Option<(Option<PathBuf>, Rope, FSTransport)>;
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

    fn editor_mut_data(
        &self,
        panel: usize,
        editor_id: usize,
    ) -> Option<(Option<PathBuf>, Rope, FSTransport)> {
        let app_state = self.read();
        let panel: &Panel = app_state.panel(panel);
        let editor = panel.tab(editor_id).as_text_editor();
        editor.map(|editor| {
            (
                editor.path().cloned(),
                editor.rope.clone(),
                editor.transport.clone(),
            )
        })
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
    // Only affects the file explorer
    FileExplorer,
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

#[derive(Clone, Default, PartialEq, Copy)]
pub enum EditorSidePanel {
    #[default]
    FileExplorer,
}

pub struct AppState {
    pub previous_focused_view: Option<EditorView>,
    pub focused_view: EditorView,
    pub focused_panel: usize,
    pub panels: Vec<Panel>,
    pub font_size: f32,
    pub line_height: f32,
    pub language_servers: HashMap<String, LSPClient>,
    pub lsp_sender: LspStatusSender,
    pub side_panel: Option<EditorSidePanel>,
    pub file_explorer_folders: Vec<TreeItem>,
    pub font_collection: FontCollection,
}

impl AppState {
    pub fn new(lsp_sender: LspStatusSender) -> Self {
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::default(), "Jetbrains Mono");

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
            file_explorer_folders: Vec::new(),
            font_collection,
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

    pub fn set_fontsize(&mut self, font_size: f32) {
        self.font_size = font_size;

        for panel in &mut self.panels {
            for tab in &mut panel.tabs {
                if let Some(editor) = tab.as_text_editor_mut() {
                    editor.measure_longest_line(font_size, &self.font_collection);
                }
            }
        }
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
            let language_id = text_editor.editor_type.language_id();
            let language_server_id = language_id.language_server();

            // Only if it ever hard LSP support
            if let Some(language_server_id) = language_server_id {
                let language_server = self.language_servers.get_mut(language_server_id);

                // And there was an actual language server running
                if let Some(language_server) = language_server {
                    let file_uri = text_editor.uri();
                    if let Some(file_uri) = file_uri {
                        language_server.close_file(file_uri);
                    }
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

    pub fn lsp(&self, lsp_config: &LspConfig) -> Option<&LSPClient> {
        self.language_servers.get(&lsp_config.language_server)
    }

    pub fn insert_lsp_client(&mut self, language_server: String, client: LSPClient) {
        self.language_servers.insert(language_server, client);
    }

    pub async fn get_or_create_lsp_client(
        mut radio: RadioAppState,
        lsp_config: &LspConfig,
    ) -> LSPClient {
        let server = radio.read().lsp(lsp_config).cloned();
        match server {
            Some(server) => server,
            None => {
                let lsp_sender = radio.read().lsp_sender.clone();
                let client = create_lsp_client(lsp_config.clone(), lsp_sender).await;
                radio
                    .write_channel(Channel::Global)
                    .insert_lsp_client(lsp_config.language_server.clone(), client.clone());
                client
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

    pub fn open_file(
        &mut self,
        path: PathBuf,
        root_path: PathBuf,
        clipboard: UseClipboard,
        content: String,
        transport: FSTransport,
        font_size: f32,
        font_collection: &FontCollection,
    ) {
        self.push_tab(
            PanelTab::TextEditor(EditorData::new(
                EditorType::FS { path, root_path },
                Rope::from(content),
                (0, 0),
                clipboard,
                transport,
                font_size,
                font_collection,
            )),
            self.focused_panel,
            true,
        );
    }

    pub fn open_folder(&mut self, item: TreeItem) {
        self.file_explorer_folders.push(item)
    }
}
