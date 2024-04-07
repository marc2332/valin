use crate::state::{Channel, RadioAppState};

use super::EditorCommand;

#[derive(Clone)]
pub struct FontSizeCommand(pub RadioAppState);

impl EditorCommand for FontSizeCommand {
    fn name(&self) -> &str {
        "fs"
    }

    fn run_with_args(&self, args: &str) {
        if let Ok(size) = args.parse::<f32>() {
            let mut radio_app_state = self.0;
            radio_app_state
                .write_channel(Channel::AllTabs)
                .set_fontsize(size);
        }
    }
}
