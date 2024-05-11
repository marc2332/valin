use std::fmt::Display;

#[derive(Clone, Default, PartialEq, Copy)]
pub enum EditorView {
    #[default]
    Panels,
    FilesExplorer,
    Commander,
}

impl Display for EditorView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Panels => f.write_str("Panels"),
            Self::FilesExplorer => f.write_str("Files Explorer"),
            Self::Commander => f.write_str("Commander"),
        }
    }
}
