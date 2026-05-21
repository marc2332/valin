use std::fmt::Display;

#[derive(Clone, Default, PartialEq, Copy, Debug)]
pub enum EditorView {
    #[default]
    Panels,
    FilesExplorer,
    Commander,
    FileSearch,
}

impl EditorView {
    pub fn is_popup(&self) -> bool {
        matches!(self, Self::Commander | Self::FileSearch)
    }
}

impl Display for EditorView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Panels => f.write_str("Panels"),
            Self::FilesExplorer => f.write_str("Files Explorer"),
            Self::Commander => f.write_str("Commander"),
            Self::FileSearch => f.write_str("File Search"),
        }
    }
}
