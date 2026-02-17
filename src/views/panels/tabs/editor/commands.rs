use freya::prelude::spawn;
use smol::{fs::OpenOptions, io::AsyncWriteExt};

use crate::{
    constants::{BASE_FONT_SIZE, MAX_FONT_SIZE},
    state::{AppStateUtils, Channel, CommandRunContext, EditorCommand, RadioAppState},
};

use crate::views::panels::tabs::editor::utils::AppStateEditorUtils;

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
        let mut radio = self.0;
        let mut app_state = radio.write_channel(Channel::AllTabs);
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
        let mut radio = self.0;
        let mut app_state = radio.write_channel(Channel::AllTabs);
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
        let mut radio = self.0;
        let active_tab = radio.get_active_tab();

        if let Some(active_tab) = active_tab {
            let editor_data = radio.read().editor_tab_data(active_tab);

            if let Some((Some(file_path), rope, transport)) = editor_data {
                spawn(async move {
                    let mut writer = transport
                        .open(&file_path, OpenOptions::new().write(true).truncate(true))
                        .await
                        .unwrap();
                    let bytes: Vec<u8> = rope.borrow().bytes().collect();
                    let _ = writer.write_all(&bytes).await;
                    let mut app_state = radio.write_channel(Channel::follow_tab(active_tab));
                    let editor_tab = app_state.editor_tab_mut(active_tab);
                    editor_tab.editor.data.mark_as_saved()
                });
            }
        }
    }
}
