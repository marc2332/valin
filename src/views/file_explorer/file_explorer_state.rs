use freya::prelude::*;

use super::file_explorer_ui::ExplorerItem;

pub struct FileExplorerState {
    pub folders: Vec<ExplorerItem>,
    pub focus_id: AccessibilityId,
}

impl FileExplorerState {
    pub fn new() -> Self {
        Self {
            folders: Vec::new(),
            focus_id: Focus::new_id(),
        }
    }

    pub fn focus(&self) {
        Focus::new_for_id(self.focus_id).request_focus();
    }

    pub fn open_folder(&mut self, item: ExplorerItem) {
        self.folders.push(item)
    }
}
