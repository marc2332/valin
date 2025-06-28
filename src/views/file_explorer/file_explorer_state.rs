use freya::{
    core::accessibility::AccessibilityFocusStrategy,
    hooks::{UseFocus, UsePlatform},
    prelude::AccessibilityId,
};

use super::file_explorer_ui::ExplorerItem;

pub struct FileExplorerState {
    pub folders: Vec<ExplorerItem>,
    pub focus_id: AccessibilityId,
}

impl FileExplorerState {
    pub fn new() -> Self {
        Self {
            folders: Vec::new(),
            focus_id: UseFocus::new_id(),
        }
    }

    pub fn focus(&self) {
        let platform = UsePlatform::current();
        platform.request_focus(AccessibilityFocusStrategy::Node(self.focus_id));
    }

    pub fn open_folder(&mut self, item: ExplorerItem) {
        self.folders.push(item)
    }
}
