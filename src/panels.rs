use crate::use_editable::EditorData;

#[derive(Clone)]
pub enum PanelTab {
    TextEditor(EditorData),
    Config,
}

impl PanelTab {
    pub fn get_data(&self) -> (String, String) {
        match self {
            PanelTab::Config => ("config".to_string(), "Config".to_string()),
            PanelTab::TextEditor(editor) => (
                editor.path().to_str().unwrap().to_owned(),
                editor
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
            ),
        }
    }

    pub fn as_text_editor(&self) -> Option<&EditorData> {
        if let PanelTab::TextEditor(editor_data) = self {
            Some(editor_data)
        } else {
            None
        }
    }

    pub fn as_text_editor_mut(&mut self) -> Option<&mut EditorData> {
        if let PanelTab::TextEditor(editor_data) = self {
            Some(editor_data)
        } else {
            None
        }
    }
}

#[derive(Clone, Default)]
pub struct Panel {
    pub active_tab: Option<usize>,
    pub tabs: Vec<PanelTab>,
}

impl Panel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active_tab(&self) -> Option<usize> {
        self.active_tab
    }

    pub fn tab(&self, editor: usize) -> &PanelTab {
        &self.tabs[editor]
    }

    pub fn tab_mut(&mut self, editor: usize) -> &mut PanelTab {
        &mut self.tabs[editor]
    }

    pub fn tabs(&self) -> &[PanelTab] {
        &self.tabs
    }

    pub fn set_active_tab(&mut self, active_tab: usize) {
        self.active_tab = Some(active_tab);
    }
}

#[derive(Clone)]
pub struct PanelsManager {
    pub is_focused: bool,
    pub focused_panel: usize,
    pub panes: Vec<Panel>,
    pub font_size: f32,
    pub line_height: f32,
}

impl Default for PanelsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PanelsManager {
    pub fn new() -> Self {
        Self {
            is_focused: true,
            focused_panel: 0,
            panes: vec![Panel::new()],
            font_size: 17.0,
            line_height: 1.2,
        }
    }

    pub fn set_fontsize(&mut self, fontsize: f32) {
        self.font_size = fontsize;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    pub fn is_focused(&self) -> bool {
        self.is_focused
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
        let opened_tab = self.panes[panel]
            .tabs
            .iter()
            .enumerate()
            .find(|(_, t)| t.get_data().0 == tab.get_data().0);

        if let Some((tab_index, _)) = opened_tab {
            if focus {
                self.focused_panel = panel;
                self.panes[panel].active_tab = Some(tab_index);
            }
        } else {
            self.panes[panel].tabs.push(tab);

            if focus {
                self.focused_panel = panel;
                self.panes[panel].active_tab = Some(self.panes[panel].tabs.len() - 1);
            }
        }
    }

    pub fn close_editor(&mut self, panel: usize, editor: usize) {
        if let Some(active_tab) = self.panes[panel].active_tab {
            let prev_editor = editor > 0;
            let next_editor = self.panes[panel].tabs.get(editor + 1).is_some();
            if active_tab == editor {
                self.panes[panel].active_tab = if next_editor {
                    Some(editor)
                } else if prev_editor {
                    Some(editor - 1)
                } else {
                    None
                };
            } else if active_tab >= editor {
                self.panes[panel].active_tab = Some(active_tab - 1);
            }
        }

        self.panes[panel].tabs.remove(editor);
    }

    pub fn push_panel(&mut self, panel: Panel) {
        self.panes.push(panel);
    }

    pub fn panels(&self) -> &[Panel] {
        &self.panes
    }

    pub fn panel(&self, panel: usize) -> &Panel {
        &self.panes[panel]
    }

    pub fn panel_mut(&mut self, panel: usize) -> &mut Panel {
        &mut self.panes[panel]
    }

    pub fn set_focused_panel(&mut self, panel: usize) {
        self.focused_panel = panel;
    }

    pub fn close_panel(&mut self, panel: usize) {
        if self.panes.len() > 1 {
            self.panes.remove(panel);
            if self.focused_panel > 0 {
                self.focused_panel -= 1;
            }
        }
    }
}
