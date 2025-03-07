use std::{collections::HashMap, vec};

use dioxus_clipboard::prelude::UseClipboard;
use dioxus_radio::prelude::{Radio, RadioChannel};
use freya::core::accessibility::AccessibilityFocusStrategy;
use freya_hooks::UsePlatform;
use skia_safe::{textlayout::FontCollection, FontMgr};
use tracing::info;

use crate::{
    fs::FSTransport,
    lsp::{create_lsp_client, LSPClient, LspConfig},
    views::file_explorer::file_explorer_state::FileExplorerState,
    LspStatusSender,
};

use super::{AppSettings, EditorView, Panel, PanelTab};

pub type RadioAppState = Radio<AppState, Channel>;

pub trait AppStateUtils {
    fn get_focused_data(&self) -> (usize, Option<usize>);
}

impl AppStateUtils for RadioAppState {
    fn get_focused_data(&self) -> (usize, Option<usize>) {
        let app_state = self.read();
        (
            app_state.focused_panel,
            app_state.panel(app_state.focused_panel).active_tab,
        )
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Channel {
    /// Affects global components
    Global,
    /// Affects all tabs
    AllTabs,
    /// Affects individual tab
    Tab {
        panel_index: usize,
        tab_index: usize,
    },
    /// Only affects the active tab
    ActiveTab,
    /// Affects the settings
    Settings,
    // Only affects the file explorer
    FileExplorer,
    // Affects nothing
    Void,
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
                                .map(move |(tab_index, _)| Self::Tab {
                                    panel_index,
                                    tab_index,
                                })
                        })
                        .collect::<Vec<Self>>(),
                );

                channels
            }
            Self::Tab {
                panel_index,
                tab_index,
            } => {
                let mut channels = vec![self];
                if app_state.focused_panel == panel_index {
                    let panel = app_state.panel(panel_index);
                    if let Some(active_tab) = panel.active_tab {
                        if active_tab == tab_index {
                            channels.push(Self::ActiveTab);
                        }
                    }
                }
                channels
            }
            Self::Settings => {
                let mut channels = vec![self];
                channels.extend(Channel::AllTabs.derive_channel(app_state));
                channels
            }
            Self::Global => {
                let mut channels = vec![self];
                channels.push(Self::ActiveTab);
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
            tab_index: editor,
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
    pub settings: AppSettings,
    pub language_servers: HashMap<String, LSPClient>,
    pub lsp_sender: LspStatusSender,
    pub side_panel: Option<EditorSidePanel>,
    pub default_transport: FSTransport,
    pub font_collection: FontCollection,
    pub clipboard: UseClipboard,

    pub file_explorer: FileExplorerState,
}

impl AppState {
    pub fn new(
        lsp_sender: LspStatusSender,
        default_transport: FSTransport,
        clipboard: UseClipboard,
    ) -> Self {
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::default(), "Jetbrains Mono");

        Self {
            previous_focused_view: None,
            focused_view: EditorView::default(),
            focused_panel: 0,
            panels: vec![Panel::new()],
            settings: AppSettings::load(),
            language_servers: HashMap::default(),
            lsp_sender,
            side_panel: Some(EditorSidePanel::default()),
            default_transport,
            font_collection,
            clipboard,

            file_explorer: FileExplorerState::new(),
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

    pub fn set_settings(&mut self, settins: AppSettings) {
        self.settings = settins;
        self.apply_settings();
    }

    pub fn set_fontsize(&mut self, font_size: f32) {
        self.settings.editor.font_size = font_size;
        self.apply_settings()
    }

    /// There are a few things that need to revaluated when the settings are changed
    pub fn apply_settings(&mut self) {
        for panel in &mut self.panels {
            for tab in &mut panel.tabs {
                tab.on_settings_changed(&self.settings, &self.font_collection)
            }
        }
    }

    pub fn set_focused_view(&mut self, focused_view: EditorView) {
        if !self.focused_view.is_popup() {
            self.previous_focused_view = Some(self.focused_view);
        }

        self.focused_view = focused_view;

        match focused_view {
            EditorView::Panels => {
                self.focus_tab(
                    self.focused_panel,
                    self.panels[self.focused_panel].active_tab,
                );
            }
            EditorView::FilesExplorer => {
                self.file_explorer.focus();
            }
            _ => {}
        }
    }

    pub fn focused_view(&self) -> &EditorView {
        &self.focused_view
    }

    pub fn set_focused_view_to_previous(&mut self) {
        if let Some(previous_focused_view) = self.previous_focused_view {
            self.focused_view = previous_focused_view;
            self.previous_focused_view = None;
        }

        match self.focused_view {
            EditorView::Panels => {
                self.focus_tab(
                    self.focused_panel,
                    self.panels[self.focused_panel].active_tab,
                );
            }
            EditorView::FilesExplorer => {
                self.file_explorer.focus();
            }
            _ => {}
        }
    }

    pub fn font_size(&self) -> f32 {
        self.settings.editor.font_size
    }

    pub fn line_height(&self) -> f32 {
        self.settings.editor.line_height
    }

    pub fn focused_panel(&self) -> usize {
        self.focused_panel
    }

    pub fn push_tab(&mut self, tab: impl PanelTab + 'static, panel: usize, focus: bool) {
        let opened_tab = self.panels[panel]
            .tabs
            .iter()
            .enumerate()
            .find(|(_, t)| t.get_data().id == tab.get_data().id);

        if let Some((tab_index, _)) = opened_tab {
            if focus {
                self.focused_panel = panel;
                self.focus_tab(panel, Some(tab_index));
            }
        } else {
            self.panels[panel].tabs.push(Box::new(tab));

            if focus {
                self.focused_panel = panel;
                self.focus_tab(panel, Some(self.panels[panel].tabs.len() - 1));
            }
        }

        if focus {
            self.focused_view = EditorView::Panels;
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
                self.focus_tab(
                    panel,
                    if next_tab {
                        Some(tab)
                    } else if prev_tab {
                        Some(tab - 1)
                    } else {
                        None
                    },
                );
            } else if active_tab >= tab {
                self.focus_tab(panel, Some(active_tab - 1));
            }
        }

        info!(
            "Closed tab [panel={panel}] [tab={}]",
            self.panels[panel].tabs.len()
        );

        let mut panel_tab = self.panels[panel].tabs.remove(tab);
        panel_tab.on_close(self);
    }

    pub fn focus_tab(&mut self, panel_i: usize, tab: Option<usize>) {
        self.panels[panel_i].active_tab = tab;
        if let Some(tab) = tab {
            let platform = UsePlatform::new();
            let tab = self.panels[panel_i].tab(tab);
            platform.focus(AccessibilityFocusStrategy::Node(tab.get_data().focus_id));
        }
    }

    pub fn close_active_tab(&mut self) {
        let panel = self.focused_panel;
        if let Some(active_tab) = self.panels[panel].active_tab {
            self.close_tab(panel, active_tab);
        }
    }

    pub fn close_active_panel(&mut self) {
        self.close_panel(self.focused_panel);
    }

    pub fn focus_previous_panel(&mut self) {
        if self.focused_panel > 0 {
            self.set_focused_panel(self.focused_panel - 1);
        }
    }

    pub fn focus_next_panel(&mut self) {
        if self.focused_panel < self.panels.len() - 1 {
            self.set_focused_panel(self.focused_panel + 1);
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
}
