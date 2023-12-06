use crate::hooks::SharedEditorManager;

use super::EditorCommand;

#[derive(Clone)]
pub struct FontSizeCommand(pub SharedEditorManager);

impl EditorCommand for FontSizeCommand {
    fn name(&self) -> &str {
        "fs"
    }

    fn run_with_args(&self, args: &str) {
        if let Ok(size) = args.parse::<f32>() {
            self.0.global_write().set_fontsize(size);
        }
    }
}
