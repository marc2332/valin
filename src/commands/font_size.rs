use crate::editor_manager::{Channel, RadioManager};

use super::EditorCommand;

#[derive(Clone)]
pub struct FontSizeCommand(pub RadioManager);

impl EditorCommand for FontSizeCommand {
    fn name(&self) -> &str {
        "fs"
    }

    fn run_with_args(&self, args: &str) {
        if let Ok(size) = args.parse::<f32>() {
            let mut radio_manager = self.0;
            radio_manager.write_channel(Channel::All).set_fontsize(size);
        }
    }
}
