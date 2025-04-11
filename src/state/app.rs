use std::{collections::HashMap, vec};

use dioxus_clipboard::prelude::UseClipboard;
use dioxus_radio::prelude::{Radio, RadioChannel};
use freya::{core::accessibility::AccessibilityFocusStrategy, hooks::UsePlatform};
use skia_safe::{textlayout::FontCollection, FontMgr};
use tracing::info;

use crate::{
    fs::FSTransport,
    lsp::{LSPClient, LspConfig},
    views::file_explorer::file_explorer_state::FileExplorerState,
    LspStatusSender,
};

use super::{AppSettings, EditorView, Panel, PanelTab, TabId};

pub type RadioAppState = Radio<AppState, Channel>;

pub trait AppStateUtils {
    fn get_active_tab(&self) -> Option<TabId>;
}

impl AppStateUtils for RadioAppState {
    fn get_active_tab(&self) -> Option<TabId> {
        let app_state = self.read();
        app_state.panel(app_state.focused_panel).active_tab
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum Channel {
    /// Affects global components
    Global,
    /// Affects all tabs
    AllTabs,
    /// Affects individual tab
    Tab {
        tab_id: TabId,
    },
    /// Only affects the active tab
    ActiveTab,
    /// Affects the settings
    Settings,
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
                        .tabs
                        .keys()
                        .map(move |tab_id| Self::Tab { tab_id: *tab_id })
                        .collect::<Vec<Self>>(),
                );

                channels
            }
            Self::Tab { tab_id } => {
                let mut channels = vec![self];
                for (panel_index, panel) in app_state.panels.iter().enumerate() {
                    if app_state.focused_panel == panel_index {
                        if let Some(active_tab) = panel.active_tab {
                            if active_tab == tab_id {
                                channels.push(Self::ActiveTab);
                            }
                        }
                    }
                }

                channels
            }
            Self::Settings => {
                let mut channels = vec![self];
                channels.extend(Channel::Global.derive_channel(app_state));
                channels
            }
            Self::Global => {
                let mut channels = vec![self];
                channels.extend(Channel::AllTabs.derive_channel(app_state));
                channels
            }
            _ => vec![self],
        }
    }
}

impl Channel {
    pub fn follow_tab(tab_id: TabId) -> Self {
        Self::Tab { tab_id }
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
    pub tabs: HashMap<TabId, Box<dyn PanelTab>>,
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
            tabs: HashMap::new(),
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
        for tab in self.tabs.values_mut() {
            tab.on_settings_changed(&self.settings, &self.font_collection)
        }
    }

    pub fn focus_view(&mut self, view: EditorView) {
        if !self.focused_view.is_popup() {
            self.previous_focused_view = Some(self.focused_view);
        }

        self.focused_view = view;

        match view {
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

    pub fn focus_previous_view(&mut self) {
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

    #[allow(clippy::borrowed_box)]
    pub fn tab(&self, tab_id: &TabId) -> &Box<dyn PanelTab> {
        self.tabs.get(tab_id).unwrap()
    }

    #[allow(clippy::borrowed_box)]
    pub fn tab_mut(&mut self, tab_id: &TabId) -> &mut Box<dyn PanelTab> {
        self.tabs.get_mut(tab_id).unwrap()
    }

    fn get_tab_if_exists(&self, tab: &impl PanelTab) -> Option<TabId> {
        self.tabs.iter().find_map(|(other_tab_id, other_tab)| {
            if other_tab.get_data().content_id == tab.get_data().content_id {
                Some(*other_tab_id)
            } else {
                None
            }
        })
    }

    // Push a [PanelTab] to a given panel index, return true if it didnt exist yet.
    pub fn push_tab(&mut self, tab: impl PanelTab + 'static, panel_index: usize) -> bool {
        let opened_tab = self.get_tab_if_exists(&tab);

        if let Some(tab_id) = opened_tab {
            // Focus the already open tab with the same content id
            self.focused_panel = panel_index;
            self.focus_tab(panel_index, Some(tab_id));
        } else {
            // Register the new tab
            self.panels[panel_index].tabs.push(tab.get_data().id);
            self.tabs.insert(tab.get_data().id, Box::new(tab));

            // Focus the new tab
            self.focused_panel = panel_index;
            self.focus_tab(panel_index, self.panels[panel_index].tabs.last().cloned());
        }

        self.focused_view = EditorView::Panels;

        info!(
            "Opened/Focused tab [panel={panel_index}] [tab={}]",
            self.panels[panel_index].tabs.len()
        );

        opened_tab.is_none()
    }

    pub fn close_tab(&mut self, tab_id: TabId) {
        let (panel_index, panel) = self
            .panels
            .iter()
            .enumerate()
            .find(|(_, panel)| panel.tabs.contains(&tab_id))
            .unwrap();
        if let Some(active_tab) = panel.active_tab {
            if active_tab == tab_id {
                let tab_index = panel.tabs.iter().position(|tab| *tab == tab_id).unwrap();
                self.focus_tab(
                    panel_index,
                    if let Some(next_tab) = panel.tabs.get(tab_index + 1) {
                        Some(*next_tab)
                    } else if tab_index > 0 {
                        panel.tabs.get(tab_index - 1).copied()
                    } else {
                        None
                    },
                );
            }
        }

        let mut panel_tab = self.tabs.remove(&tab_id).unwrap();
        panel_tab.on_close(self);

        let panel = self
            .panels
            .iter_mut()
            .find(|panel| panel.tabs.contains(&tab_id))
            .unwrap();
        panel.tabs.retain(|tab| *tab != tab_id);

        info!("Closed tab [panel={panel_index}] [tab={tab_id:?}]",);
    }

    pub fn focus_tab(&mut self, panel_index: usize, tab_id: Option<TabId>) {
        self.panels[panel_index].active_tab = tab_id;
        if let Some(tab_id) = tab_id {
            let platform = UsePlatform::current();
            let tab = self.tab(&tab_id);
            platform.focus(AccessibilityFocusStrategy::Node(tab.get_data().focus_id));
        }
    }

    pub fn close_active_tab(&mut self) {
        let panel = self.focused_panel;
        if let Some(active_tab) = self.panels[panel].active_tab {
            self.close_tab(active_tab);
        }
    }

    pub fn close_active_panel(&mut self) {
        self.close_panel(self.focused_panel);
    }

    pub fn focus_previous_panel(&mut self) {
        if self.focused_panel > 0 {
            self.focus_panel(self.focused_panel - 1);
        }
    }

    pub fn focus_next_panel(&mut self) {
        if self.focused_panel < self.panels.len() - 1 {
            self.focus_panel(self.focused_panel + 1);
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

    pub fn focus_panel(&mut self, panel: usize) {
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
        info!("Registered language server '{language_server}'");
        self.language_servers.insert(language_server, client);
    }
}
