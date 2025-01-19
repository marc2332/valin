use freya::prelude::spawn;
use tokio::fs::OpenOptions;

use crate::{
    constants::{BASE_FONT_SIZE, MAX_FONT_SIZE},
    state::{AppStateUtils, Channel, CommandRunContext, EditorCommand, RadioAppState},
};

use crate::tabs::editor::utils::AppStateEditorUtils;

#[derive(Clone)]
pub struct IncreaseFontSizeCommand(pub RadioAppState);

impl IncreaseFontSizeCommand {
    pub fn id() -> &'static str {
        "increase-editor-font-size"
    }
}

impl EditorCommand for IncreaseFontSizeCommand {
    fn matches(&self, input: &str) -> bool {
        self.text().to_lowercase().contains(&input.to_lowercase())
    }

    fn id(&self) -> &str {
        Self::id()
    }

    fn text(&self) -> &str {
        "Increase Font Size"
    }

    fn run(&self, _ctx: &mut CommandRunContext) {
        let mut radio_app_state = self.0;
        let mut app_state = radio_app_state.write_channel(Channel::AllTabs);
        let font_size = app_state.font_size();
        app_state.set_fontsize((font_size + 2.0).clamp(BASE_FONT_SIZE, MAX_FONT_SIZE));
    }
}

#[derive(Clone)]
pub struct DecreaseFontSizeCommand(pub RadioAppState);

impl DecreaseFontSizeCommand {
    pub fn id() -> &'static str {
        "decrease-editor-font-size"
    }
}

impl EditorCommand for DecreaseFontSizeCommand {
    fn matches(&self, input: &str) -> bool {
        self.text().to_lowercase().contains(&input.to_lowercase())
    }

    fn id(&self) -> &str {
        Self::id()
    }

    fn text(&self) -> &str {
        "Decrease Font Size"
    }

    fn run(&self, _ctx: &mut CommandRunContext) {
        let mut radio_app_state = self.0;
        let mut app_state = radio_app_state.write_channel(Channel::AllTabs);
        let font_size = app_state.font_size();
        app_state.set_fontsize((font_size - 2.0).clamp(BASE_FONT_SIZE, MAX_FONT_SIZE));
    }
}

#[derive(Clone)]
pub struct SaveFileCommand(pub RadioAppState);

impl SaveFileCommand {
    pub fn id() -> &'static str {
        "save-file"
    }
}

impl EditorCommand for SaveFileCommand {
    fn matches(&self, input: &str) -> bool {
        self.text().to_lowercase().contains(&input.to_lowercase())
    }

    fn id(&self) -> &str {
        Self::id()
    }

    fn text(&self) -> &str {
        "Save File"
    }

    fn run(&self, _ctx: &mut CommandRunContext) {
        let mut radio_app_state = self.0;
        let (panel, active_tab) = radio_app_state.get_focused_data();

        if let Some(active_tab) = active_tab {
            let editor_data = {
                let app_state = radio_app_state.read();
                app_state.editor_tab_data(panel, active_tab)
            };

            if let Some((Some(file_path), rope, transport)) = editor_data {
                spawn(async move {
                    let writer = transport
                        .open(&file_path, OpenOptions::new().write(true).truncate(true))
                        .await
                        .unwrap();
                    let std_writer = writer.into_std().await;
                    rope.borrow_mut().write_to(std_writer).unwrap();
                    let mut app_state =
                        radio_app_state.write_channel(Channel::follow_tab(panel, active_tab));
                    let editor_tab = app_state.try_editor_tab_mut(panel, active_tab);
                    if let Some(editor_tab) = editor_tab {
                        editor_tab.editor.mark_as_saved()
                    }
                });
            }
        }
    }
}
