use super::EditorData;

pub enum PanelTab {
    TextEditor(EditorData),
    Config,
    Welcome,
}

impl PanelTab {
    pub fn get_data(&self) -> PanelTabData {
        match self {
            PanelTab::Config => PanelTabData {
                id: "config".to_string(),
                title: "Config".to_string(),
                edited: false,
            },
            PanelTab::TextEditor(editor) => {
                let (title, id) = editor.editor_type.title_and_id();
                PanelTabData {
                    id,
                    title,
                    edited: editor.is_edited(),
                }
            }
            PanelTab::Welcome => PanelTabData {
                id: "welcome".to_string(),
                title: "Welcome".to_string(),
                edited: false,
            },
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

#[derive(PartialEq, Eq)]
pub struct PanelTabData {
    pub edited: bool,
    pub title: String,
    pub id: String,
}

#[derive(Default)]
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
