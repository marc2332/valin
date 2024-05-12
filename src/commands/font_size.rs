use crate::{constants::{BASE_FONT_SIZE, MAX_FONT_SIZE}, state::{Channel, RadioAppState}};

use super::EditorCommand;

#[derive(Clone)]
pub struct IncreaseFontSize(pub RadioAppState);

impl IncreaseFontSize {
    pub fn id() -> &'static str {
        "increase-editor-font-size"
    }
}

impl EditorCommand for IncreaseFontSize {
    fn matches(&self, input: &str) -> bool {
        self.text().to_lowercase().contains(&input.to_lowercase())
    }

    fn id(&self) -> &str {
        Self::id()
    }

    fn text(&self) -> &str {
        "Increase Font Size"
    }

    fn run(&self) {
        let mut radio_app_state = self.0;
        let mut app_state = radio_app_state.write_channel(Channel::AllTabs);
        let font_size = app_state.font_size();
        app_state.set_fontsize((font_size + 4.0).clamp(BASE_FONT_SIZE, MAX_FONT_SIZE));
    }
}


#[derive(Clone)]
pub struct DecreaseFontSize(pub RadioAppState);

impl DecreaseFontSize {
    pub fn id() -> &'static str {
        "decrease-editor-font-size"
    }
}

impl EditorCommand for DecreaseFontSize {
    fn matches(&self, input: &str) -> bool {
        self.text().to_lowercase().contains(&input.to_lowercase())
    }

    fn id(&self) -> &str {
        Self::id()
    }

    fn text(&self) -> &str {
        "Decrease Font Size"
    }

    fn run(&self) {
        let mut radio_app_state = self.0;
        let mut app_state = radio_app_state.write_channel(Channel::AllTabs);
        let font_size = app_state.font_size();
        app_state.set_fontsize((font_size - 4.0).clamp(BASE_FONT_SIZE, MAX_FONT_SIZE));
    }
}
