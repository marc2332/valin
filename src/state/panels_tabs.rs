use std::{
    any::Any,
    fmt::Display,
    sync::atomic::{AtomicU64, Ordering},
};

use freya::prelude::*;
use skia_safe::textlayout::FontCollection;

use super::{AppSettings, AppState};

pub trait PanelTab {
    fn on_close(&mut self, _app_state: &mut AppState) {}

    fn on_settings_changed(
        &mut self,
        _app_settings: &AppSettings,
        _font_collection: &FontCollection,
    ) {
    }

    fn get_data(&self) -> PanelTabData;

    fn render(&self) -> fn(TabProps) -> Element;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Props, Clone, PartialEq)]
pub struct TabProps {
    pub tab_id: TabId,
}

static TAB_IDS: AtomicU64 = AtomicU64::new(0);

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct TabId(u64);

impl TabId {
    pub fn new() -> Self {
        let n = TAB_IDS.fetch_add(1, Ordering::Relaxed);
        Self(n)
    }
}

impl Display for TabId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

#[derive(PartialEq, Eq)]
pub struct PanelTabData {
    pub edited: bool,
    pub title: String,
    pub content_id: String,
    pub id: TabId,
    pub focus_id: AccessibilityId,
}

#[derive(Default)]
pub struct Panel {
    pub active_tab: Option<TabId>,
    pub tabs: Vec<TabId>,
}

impl Panel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active_tab(&self) -> Option<TabId> {
        self.active_tab
    }

    pub fn set_active_tab(&mut self, active_tab: TabId) {
        self.active_tab = Some(active_tab);
    }
}
