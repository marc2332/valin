use crate::state::{Channel, Panel, RadioAppState};

use super::EditorCommand;

#[derive(Clone)]
pub struct SplitCommand(pub RadioAppState);

impl EditorCommand for SplitCommand {
    fn name(&self) -> &str {
        "split"
    }

    fn run_with_args(&self, args: &str) {
        #[allow(clippy::single_match)]
        match args {
            "panel" => {
                let mut radio_app_state = self.0;
                let len_panels = radio_app_state.read().panels().len();
                let mut app_state = radio_app_state.write_channel(Channel::Global);
                app_state.push_panel(Panel::new());
                app_state.set_focused_panel(len_panels - 1);
            }
            _ => {}
        }
    }
}
