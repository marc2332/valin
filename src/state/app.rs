use std::collections::HashMap;

use freya::prelude::*;
use freya::radio::{Radio, RadioChannel};
use futures_channel::mpsc::UnboundedSender;
use tracing::info;

use crate::{fs::FSTransport, views::file_explorer::file_explorer_state::FileExplorerState};

use super::{AppSettings, EditorView, FileIcons, PanelId, PanelTab, TabId, TabSwitcherState};

pub type RadioAppState = Radio<AppState, Channel>;

pub trait AppStateUtils {
    fn get_active_tab(&self) -> Option<TabId>;
}

impl AppStateUtils for RadioAppState {
    fn get_active_tab(&self) -> Option<TabId> {
        let app_state = self.read();
        let panel_id = app_state.focused_panel?;
        app_state
            .panel_tree
            .as_ref()?
            .panel(&panel_id)?
            .active_tab_id
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
            Self::AllTabs => [self, Self::ActiveTab]
                .into_iter()
                .chain(app_state.tabs.keys().map(|&tab_id| Self::Tab { tab_id }))
                .collect(),
            Self::Tab { tab_id } => {
                let is_active_in_focused = app_state
                    .focused_panel
                    .and_then(|pid| app_state.panel_tree.as_ref()?.panel(&pid))
                    .map(|panel| panel.active_tab_id == Some(tab_id))
                    .unwrap_or(false);
                if is_active_in_focused {
                    vec![self, Self::ActiveTab]
                } else {
                    vec![self]
                }
            }
            Self::Settings => vec![self, Self::Global],
            Self::Global => std::iter::once(self)
                .chain(Self::AllTabs.derive_channel(app_state))
                .collect(),
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

    pub focused_panel: Option<PanelId>,
    pub panel_tree: Option<DockNode<TabId, PanelId>>,

    pub tabs: HashMap<TabId, Box<dyn PanelTab>>,
    pub tab_history: Vec<TabId>,
    pub tab_switcher: Option<TabSwitcherState>,
    pub settings: AppSettings,
    pub side_panel: Option<EditorSidePanel>,
    pub default_transport: FSTransport,

    pub file_explorer: FileExplorerState,
    pub file_icons: FileIcons,

    pub task_sender: UnboundedSender<AppTask>,
}

/// A background task.
pub enum AppTask {
    OpenFile {
        path: std::path::PathBuf,
        panel_id: PanelId,
    },
}

/// What a drag can carry onto the docking area: an existing tab, or a file path
/// (dragged in from the file explorer) to open as a new tab.
#[derive(Clone, PartialEq)]
pub enum DropValue {
    Tab(TabId),
    File(std::path::PathBuf),
}

impl From<TabId> for DropValue {
    fn from(tab_id: TabId) -> Self {
        DropValue::Tab(tab_id)
    }
}

impl AppState {
    pub fn new(default_transport: FSTransport, task_sender: UnboundedSender<AppTask>) -> Self {
        let panel_id = PanelId::new();
        Self {
            previous_focused_view: None,
            focused_view: EditorView::default(),
            focused_panel: Some(panel_id),
            panel_tree: Some(DockNode::Panel(DockPanel::new(panel_id, vec![]))),
            tabs: HashMap::new(),
            tab_history: Vec::new(),
            tab_switcher: None,
            settings: AppSettings::load(),
            side_panel: Some(EditorSidePanel::default()),
            default_transport,
            file_explorer: FileExplorerState::new(),
            file_icons: FileIcons::new(),
            task_sender,
        }
    }

    pub fn toggle_side_panel(&mut self, side_panel: EditorSidePanel) {
        self.side_panel = if self.side_panel == Some(side_panel) {
            None
        } else {
            Some(side_panel)
        };
    }

    pub fn set_settings(&mut self, settings: AppSettings) {
        self.settings = settings;
        self.apply_settings();
    }

    pub fn set_fontsize(&mut self, font_size: f32) {
        self.settings.editor.font_size = font_size;
        self.apply_settings()
    }

    pub fn apply_settings(&mut self) {
        for tab in self.tabs.values_mut() {
            tab.on_settings_changed(&self.settings)
        }
    }

    fn focus_view_inner(&mut self, view: EditorView) {
        match view {
            EditorView::Panels => {
                if let Some(panel_id) = self.focused_panel {
                    let active_tab = self
                        .panel_tree
                        .as_ref()
                        .and_then(|t| t.panel(&panel_id))
                        .and_then(|p| p.active_tab_id);
                    self.focus_tab(panel_id, active_tab);
                }
            }
            EditorView::FilesExplorer => self.file_explorer.focus(),
            _ => {}
        }
    }

    pub fn focus_view(&mut self, view: EditorView) {
        if !self.focused_view.is_popup() {
            self.previous_focused_view = Some(self.focused_view);
        }
        self.focused_view = view;
        self.focus_view_inner(view);
    }

    pub fn focus_previous_view(&mut self) {
        if self.focused_view == EditorView::TabSwitcher {
            self.tab_switcher = None;
        }
        if let Some(previous_focused_view) = self.previous_focused_view {
            self.focused_view = previous_focused_view;
            self.previous_focused_view = None;
        }
        self.focus_view_inner(self.focused_view);
    }

    pub fn font_size(&self) -> f32 {
        self.settings.editor.font_size
    }

    pub fn line_height(&self) -> f32 {
        self.settings.editor.line_height
    }

    pub fn tab(&self, tab_id: &TabId) -> &(dyn PanelTab + 'static) {
        self.tabs.get(tab_id).unwrap().as_ref()
    }

    pub fn tab_mut(&mut self, tab_id: &TabId) -> &mut (dyn PanelTab + 'static) {
        self.tabs.get_mut(tab_id).unwrap().as_mut()
    }

    /// Find an open tab by its content id.
    fn find_tab_by_content_id(&self, content_id: &str) -> Option<TabId> {
        self.tabs
            .iter()
            .find_map(|(id, tab)| (tab.get_data().content_id == content_id).then_some(*id))
    }

    /// Return all panel ids in tree-traversal order.
    pub fn panels_in_order(&self) -> Vec<PanelId> {
        fn collect(node: &DockNode<TabId, PanelId>, result: &mut Vec<PanelId>) {
            match node {
                DockNode::Panel(panel) => result.push(panel.panel_id),
                DockNode::Split { children, .. } => {
                    for child in children {
                        collect(child, result);
                    }
                }
            }
        }
        let mut result = Vec::new();
        if let Some(tree) = &self.panel_tree {
            collect(tree, &mut result);
        }
        result
    }

    /// Remove a single panel from the tree, flattening splits left with one child.
    fn remove_panel_node(&mut self, panel_id: PanelId) {
        if let Some(tree) = self.panel_tree.as_mut() {
            remove_panel_from_tree(tree, panel_id);
        }
    }

    /// Push a tab into the given panel (or the focused / first available panel).
    /// Returns `true` if the tab was newly opened, `false` if it already existed.
    pub fn push_tab(&mut self, tab: impl PanelTab + 'static, panel_id: Option<PanelId>) -> bool {
        let opened_tab = self.find_tab_by_content_id(&tab.get_data().content_id);

        if let Some(existing_id) = opened_tab {
            if let Some((existing_panel_id, _)) = self
                .panel_tree
                .as_ref()
                .and_then(|tree| tree.find_tab(&existing_id))
            {
                self.focused_panel = Some(existing_panel_id);
                self.focus_tab(existing_panel_id, Some(existing_id));
            }
            self.focused_view = EditorView::Panels;
            return false;
        }

        let tab_id = tab.get_data().id;

        let target = match panel_id.or(self.focused_panel).filter(|pid| {
            self.panel_tree
                .as_ref()
                .and_then(|t| t.panel(pid))
                .is_some()
        }) {
            Some(pid) => pid,
            None => unreachable!("There is always at least 1 panel."),
        };

        self.tabs.insert(tab_id, Box::new(tab));

        if let Some(panel) = self.panel_tree.as_mut().and_then(|t| t.panel_mut(&target)) {
            panel.tabs.push(tab_id);
            panel.active_tab_id = Some(tab_id);
        }

        self.focused_panel = Some(target);
        self.focus_tab(target, Some(tab_id));
        self.focused_view = EditorView::Panels;

        info!("Opened tab [panel={target:?}] [tab={tab_id}]",);
        true
    }

    pub fn close_tab(&mut self, tab_id: TabId) {
        let Some((panel_id, tab_pos)) = self
            .panel_tree
            .as_ref()
            .and_then(|tree| tree.find_tab(&tab_id))
        else {
            return;
        };

        let is_active = self
            .panel_tree
            .as_ref()
            .and_then(|tree| tree.panel(&panel_id))
            .map(|panel| panel.active_tab_id == Some(tab_id))
            .unwrap_or(false);

        if is_active {
            let next = self
                .panel_tree
                .as_ref()
                .and_then(|tree| tree.panel(&panel_id))
                .and_then(|panel| {
                    panel
                        .tabs
                        .get(tab_pos + 1)
                        .or_else(|| tab_pos.checked_sub(1).and_then(|i| panel.tabs.get(i)))
                        .copied()
                });
            self.focus_tab(panel_id, next);
        }

        let Some(mut panel_tab) = self.tabs.remove(&tab_id) else {
            return;
        };
        panel_tab.on_close(self);

        // Keep the panel even if it becomes empty.
        if let Some(tree) = self.panel_tree.as_mut() {
            tree.remove_tab_except(&tab_id, None);
        }

        self.tab_history.retain(|t| *t != tab_id);
        if let Some(switcher) = self.tab_switcher.as_mut() {
            switcher.order.retain(|t| *t != tab_id);
            if switcher.selected >= switcher.order.len() {
                switcher.selected = switcher.order.len().saturating_sub(1);
            }
        }

        info!("Closed tab [{tab_id:?}]");
    }

    pub fn focus_tab(&mut self, panel_id: PanelId, tab_id: Option<TabId>) {
        if let Some(panel) = self
            .panel_tree
            .as_mut()
            .and_then(|tree| tree.panel_mut(&panel_id))
        {
            panel.active_tab_id = tab_id;
        }
        if let Some(tab_id) = tab_id {
            let tab = self.tab(&tab_id);
            tab.get_data().focus_id.request_focus();
            if self.tab_switcher.is_none() {
                self.tab_history.retain(|t| *t != tab_id);
                self.tab_history.insert(0, tab_id);
            }
        }
    }

    pub fn close_active_tab(&mut self) {
        let active_tab = self
            .focused_panel
            .and_then(|pid| self.panel_tree.as_ref()?.panel(&pid))
            .and_then(|panel| panel.active_tab_id);
        if let Some(tab_id) = active_tab {
            self.close_tab(tab_id);
        }
    }

    pub fn close_active_panel(&mut self) {
        if let Some(panel_id) = self.focused_panel {
            self.close_panel(panel_id);
        }
    }

    pub fn split_focused_panel(&mut self) {
        if let Some(panel_id) = self.focused_panel {
            self.split_panel(panel_id);
        }
    }

    pub fn split_panel(&mut self, panel_id: PanelId) {
        self.split_panel_side(panel_id, Side::Right);
    }

    /// Split `panel_id` on `side` with a new empty panel and return its id.
    pub fn split_panel_side(&mut self, panel_id: PanelId, side: Side) -> PanelId {
        let new_panel_id = PanelId::new();
        let new_panel = DockPanel::new(new_panel_id, vec![]);

        if let Some(tree) = self.panel_tree.as_mut() {
            tree.split_panel(&panel_id, side, &new_panel);
        }

        self.focused_panel = Some(new_panel_id);
        self.focused_view = EditorView::Panels;
        new_panel_id
    }

    /// Move an existing tab to a drop target.
    fn move_tab(&mut self, tab: TabId, target: DropTarget<PanelId>) -> bool {
        // Panel the tab currently lives in.
        let source_panel = self
            .panel_tree
            .as_ref()
            .and_then(|tree| tree.find_tab(&tab))
            .map(|(panel_id, _)| panel_id);

        let Some(tree) = self.panel_tree.as_mut() else {
            return false;
        };

        // Panel the tab lands in.
        let destination = match target {
            DropTarget::Tab { panel_id, position } => {
                let Some(target_panel) = tree.panel_mut(&panel_id) else {
                    return false;
                };
                target_panel.insert_tab(tab, position);
                tree.remove_tab_except(&tab, Some(&panel_id));
                panel_id
            }
            DropTarget::Center(panel_id) => {
                let Some(target_panel) = tree.panel_mut(&panel_id) else {
                    return false;
                };
                target_panel.append_tab(tab);
                tree.remove_tab_except(&tab, Some(&panel_id));
                panel_id
            }
            DropTarget::Split { panel_id, side } => {
                let new_panel_id = PanelId::new();
                let new_panel = DockPanel::new(new_panel_id, vec![tab]);
                if !tree.split_panel(&panel_id, side, &new_panel) {
                    return false;
                }
                tree.remove_tab_except(&tab, Some(&new_panel_id));
                new_panel_id
            }
        };

        self.focused_panel = Some(destination);

        // Collapse the source panel if now empty.
        if let Some(source) = source_panel {
            let source_empty = self
                .panel_tree
                .as_ref()
                .and_then(|tree| tree.panel(&source))
                .map(|panel| panel.tabs.is_empty())
                .unwrap_or(false);
            if source_empty && self.panels_in_order().len() > 1 {
                self.remove_panel_node(source);
            }
        }

        true
    }

    /// Open a dropped file at a drop target.
    fn open_file_at(&mut self, path: std::path::PathBuf, target: DropTarget<PanelId>) -> bool {
        if let Some(tab_id) = self.find_tab_by_content_id(&path.to_string_lossy()) {
            return self.move_tab(tab_id, target);
        }
        let panel_id = match target {
            DropTarget::Tab { panel_id, .. } | DropTarget::Center(panel_id) => panel_id,
            DropTarget::Split { panel_id, side } => self.split_panel_side(panel_id, side),
        };
        self.task_sender
            .unbounded_send(AppTask::OpenFile { path, panel_id })
            .is_ok()
    }

    pub fn close_panel(&mut self, panel_id: PanelId) {
        let order = self.panels_in_order();
        // Prevent closing the last panel
        if order.len() <= 1 {
            return;
        }

        // Next panel to focus, the one after or else before.
        let neighbor = order.iter().position(|&id| id == panel_id).and_then(|idx| {
            order
                .get(idx + 1)
                .or_else(|| idx.checked_sub(1).and_then(|prev| order.get(prev)))
                .copied()
        });

        let tabs: Vec<TabId> = self
            .panel_tree
            .as_ref()
            .and_then(|tree| tree.panel(&panel_id))
            .map(|panel| panel.tabs.clone())
            .unwrap_or_default();

        for tab_id in tabs {
            if let Some(mut tab) = self.tabs.remove(&tab_id) {
                tab.on_close(self);
            }
            self.tab_history.retain(|t| *t != tab_id);
            if let Some(switcher) = self.tab_switcher.as_mut() {
                switcher.order.retain(|t| *t != tab_id);
            }
        }

        self.remove_panel_node(panel_id);

        if self.focused_panel == Some(panel_id) {
            self.focused_panel = neighbor;
        }

        if let Some(switcher) = self.tab_switcher.as_mut()
            && switcher.selected >= switcher.order.len()
        {
            switcher.selected = switcher.order.len().saturating_sub(1);
        }
    }

    pub fn focus_next_panel(&mut self) {
        let panels = self.panels_in_order();
        let Some(current) = self.focused_panel else {
            return;
        };
        if let Some(idx) = panels.iter().position(|&id| id == current)
            && let Some(&next_id) = panels.get(idx + 1)
        {
            self.focused_panel = Some(next_id);
            let active_tab = self
                .panel_tree
                .as_ref()
                .and_then(|t| t.panel(&next_id))
                .and_then(|p| p.active_tab_id);
            self.focus_tab(next_id, active_tab);
        }
    }

    pub fn focus_previous_panel(&mut self) {
        let panels = self.panels_in_order();
        let Some(current) = self.focused_panel else {
            return;
        };
        if let Some(idx) = panels.iter().position(|&id| id == current)
            && idx > 0
        {
            let prev_id = panels[idx - 1];
            self.focused_panel = Some(prev_id);
            let active_tab = self
                .panel_tree
                .as_ref()
                .and_then(|t| t.panel(&prev_id))
                .and_then(|p| p.active_tab_id);
            self.focus_tab(prev_id, active_tab);
        }
    }

    pub fn cycle_tab_switcher(&mut self, reverse: bool) {
        let step = |selected: usize, len: usize| {
            let delta = if reverse { len - 1 } else { 1 };
            (selected + delta) % len
        };

        if let Some(switcher) = self.tab_switcher.as_mut() {
            if !switcher.order.is_empty() {
                switcher.selected = step(switcher.selected, switcher.order.len());
            }
            return;
        }

        if self.tab_history.len() < 2 {
            return;
        }
        let order = self.tab_history.clone();
        let selected = step(0, order.len());
        self.tab_switcher = Some(TabSwitcherState { order, selected });
        self.focus_view(EditorView::TabSwitcher);
    }

    pub fn commit_tab_switcher(&mut self) {
        let Some(switcher) = self.tab_switcher.take() else {
            return;
        };
        if let Some(&tab_id) = switcher.order.get(switcher.selected)
            && let Some((panel_id, _)) = self
                .panel_tree
                .as_ref()
                .and_then(|tree| tree.find_tab(&tab_id))
        {
            self.focused_panel = Some(panel_id);
            self.focus_tab(panel_id, Some(tab_id));
        }
        self.focus_previous_view();
    }
}

impl DockingModel for AppState {
    type TabId = TabId;
    type PanelId = PanelId;
    type DropValue = DropValue;

    fn root(&self) -> Option<&DockNode<TabId, PanelId>> {
        self.panel_tree.as_ref()
    }

    fn on_drop(&mut self, value: DropValue, target: DropTarget<PanelId>) -> bool {
        match value {
            DropValue::Tab(tab) => self.move_tab(tab, target),
            DropValue::File(path) => self.open_file_at(path, target),
        }
    }

    fn set_active(&mut self, panel: PanelId, tab: TabId) -> bool {
        let Some(panel_node) = self.panel_tree.as_mut().and_then(|t| t.panel_mut(&panel)) else {
            return false;
        };
        if !panel_node.tabs.contains(&tab) {
            return false;
        }
        panel_node.active_tab_id = Some(tab);
        self.focused_panel = Some(panel);
        self.focused_view = EditorView::Panels;
        true
    }
}

/// Remove the `target` panel from the tree and flatten single-child splits.
/// Returns `true` if it was found.
fn remove_panel_from_tree(node: &mut DockNode<TabId, PanelId>, target: PanelId) -> bool {
    let DockNode::Split { children, .. } = node else {
        return false;
    };

    let removed = if let Some(pos) = children
        .iter()
        .position(|child| matches!(child, DockNode::Panel(panel) if panel.panel_id == target))
    {
        children.remove(pos);
        true
    } else {
        children
            .iter_mut()
            .any(|child| remove_panel_from_tree(child, target))
    };

    if removed && children.len() == 1 {
        *node = children.remove(0);
    }

    removed
}
