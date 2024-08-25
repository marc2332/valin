use std::path::PathBuf;

use crate::{
    fs::FSTransport,
    state::{AppState, Panel, PanelTab},
};

use super::{EditorTab, SharedRope};

pub trait AppStateEditorUtils {
    fn editor_tab(&self, panel: usize, editor_id: usize) -> &EditorTab;

    fn editor_tab_mut(&mut self, panel: usize, editor_id: usize) -> &mut EditorTab;

    fn try_editor_tab_mut(&mut self, panel: usize, editor_id: usize) -> Option<&mut EditorTab>;

    fn editor_tab_data(
        &self,
        panel: usize,
        editor_id: usize,
    ) -> Option<(Option<PathBuf>, SharedRope, FSTransport)>;
}

impl AppStateEditorUtils for AppState {
    fn editor_tab(&self, panel: usize, editor_id: usize) -> &EditorTab {
        self.panel(panel).tab(editor_id).as_text_editor().unwrap()
    }

    fn editor_tab_mut(&mut self, panel: usize, editor_id: usize) -> &mut EditorTab {
        self.panel_mut(panel)
            .tab_mut(editor_id)
            .as_text_editor_mut()
            .unwrap()
    }

    fn try_editor_tab_mut(&mut self, panel: usize, editor_id: usize) -> Option<&mut EditorTab> {
        self.panel_mut(panel)
            .tab_mut(editor_id)
            .as_text_editor_mut()
    }

    fn editor_tab_data(
        &self,
        panel: usize,
        editor_id: usize,
    ) -> Option<(Option<PathBuf>, SharedRope, FSTransport)> {
        let panel: &Panel = self.panel(panel);
        let editor = panel.tab(editor_id).as_text_editor();
        editor.map(|EditorTab { editor: data }| {
            (
                data.path().cloned(),
                data.rope.clone(),
                data.transport.clone(),
            )
        })
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
