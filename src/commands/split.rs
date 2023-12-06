use crate::hooks::{Panel, SharedEditorManager};

use super::EditorCommand;

#[derive(Clone)]
pub struct SplitCommand(pub SharedEditorManager);

impl EditorCommand for SplitCommand {
    fn name(&self) -> &str {
        "split"
    }

    fn run_with_args(&self, args: &str) {
        #[allow(clippy::single_match)]
        match args {
            "panel" => {
                let len_panels = self.0.current().panels().len();
                let mut manager = self.0.global_write();
                manager.push_panel(Panel::new());
                manager.set_focused_panel(len_panels - 1);
            }
            _ => {}
        }
    }
}
