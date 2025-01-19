use std::any::Any;

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
    pub panel_index: usize,
    pub tab_index: usize,
}

#[derive(PartialEq, Eq)]
pub struct PanelTabData {
    pub edited: bool,
    pub title: String,
    pub id: String,
    pub focus_id: AccessibilityId,
}

#[derive(Default)]
pub struct Panel {
    pub active_tab: Option<usize>,
    pub tabs: Vec<Box<dyn PanelTab>>,
}

impl Panel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active_tab(&self) -> Option<usize> {
        self.active_tab
    }

    #[allow(clippy::borrowed_box)]
    pub fn tab(&self, editor: usize) -> &Box<dyn PanelTab> {
        &self.tabs[editor]
    }

    pub fn tab_mut(&mut self, editor: usize) -> &mut Box<dyn PanelTab> {
        &mut self.tabs[editor]
    }

    pub fn tabs(&self) -> &[Box<dyn PanelTab>] {
        &self.tabs
    }

    pub fn set_active_tab(&mut self, active_tab: usize) {
        self.active_tab = Some(active_tab);
    }
}
