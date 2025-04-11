use std::path::PathBuf;

use crate::{
    fs::FSTransport,
    lsp::{LSPClient, LspConfig},
    state::{AppState, PanelTab, TabId},
};

use super::{EditorTab, SharedRope};

pub trait AppStateEditorUtils {
    fn editor_tab(&self, tab_id: TabId) -> &EditorTab;

    fn editor_tab_mut(&mut self, tab_id: TabId) -> &mut EditorTab;

    fn editor_tab_data(&self, tab_id: TabId) -> Option<(Option<PathBuf>, SharedRope, FSTransport)>;

    fn editor_tab_lsp(&self, tab_id: TabId) -> Option<LSPClient>;
}

impl AppStateEditorUtils for AppState {
    fn editor_tab(&self, tab_id: TabId) -> &EditorTab {
        self.tabs.get(&tab_id).unwrap().as_text_editor().unwrap()
    }

    fn editor_tab_mut(&mut self, tab_id: TabId) -> &mut EditorTab {
        self.tabs
            .get_mut(&tab_id)
            .unwrap()
            .as_text_editor_mut()
            .unwrap()
    }

    fn editor_tab_data(&self, tab_id: TabId) -> Option<(Option<PathBuf>, SharedRope, FSTransport)> {
        let tab = self.tabs.get(&tab_id)?.as_text_editor()?;
        Some((
            tab.editor.path().cloned(),
            tab.editor.rope.clone(),
            tab.editor.transport.clone(),
        ))
    }

    fn editor_tab_lsp(&self, tab_id: TabId) -> Option<LSPClient> {
        let editor_tab = self.editor_tab(tab_id);
        let lsp_config = LspConfig::new(editor_tab.editor.editor_type.clone())?;
        self.lsp(&lsp_config).cloned()
    }
}

pub trait TabEditorUtils {
    fn as_text_editor(&self) -> Option<&EditorTab>;

    fn as_text_editor_mut(&mut self) -> Option<&mut EditorTab>;
}

impl TabEditorUtils for Box<dyn PanelTab> {
    fn as_text_editor(&self) -> Option<&EditorTab> {
        self.as_any().downcast_ref()
    }

    fn as_text_editor_mut(&mut self) -> Option<&mut EditorTab> {
        self.as_any_mut().downcast_mut()
    }
}
