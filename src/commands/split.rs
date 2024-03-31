use crate::editor_manager::{Panel, RadioManager, SubscriptionModel};

use super::EditorCommand;

#[derive(Clone)]
pub struct SplitCommand(pub RadioManager);

impl EditorCommand for SplitCommand {
    fn name(&self) -> &str {
        "split"
    }

    fn run_with_args(&self, args: &str) {
        #[allow(clippy::single_match)]
        match args {
            "panel" => {
                let mut radio_manager = self.0;
                let len_panels = radio_manager.read().panels().len();
                let mut manager = radio_manager.write_channel(SubscriptionModel::All);
                manager.push_panel(Panel::new());
                manager.set_focused_panel(len_panels - 1);
            }
            _ => {}
        }
    }
}
