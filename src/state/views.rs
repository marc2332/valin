use std::fmt::Display;

#[derive(Clone, Default, PartialEq, Copy)]
pub enum EditorView {
    #[default]
    CodeEditor,
    FilesExplorer,
    Commander,
}

impl Display for EditorView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CodeEditor => f.write_str("Code Editor"),
            Self::FilesExplorer => f.write_str("Files Explorer"),
            Self::Commander => f.write_str("Commander"),
        }
    }
}
