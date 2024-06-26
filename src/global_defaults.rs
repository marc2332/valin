use crate::{
    state::{Channel, EditorCommand, EditorView, Panel, RadioAppState},
    tabs::settings::Settings,
};

#[allow(non_snake_case)]
pub mod GlobalDefaults {
    use freya::events::{Code, KeyboardData, Modifiers};

    use crate::state::{Channel, EditorCommands, EditorView, KeyboardShortcuts, RadioAppState};

    use super::{OpenSettingsCommand, SplitPanelCommand, ToggleCommanderCommand};

    pub fn init(
        keyboard_shorcuts: &mut KeyboardShortcuts,
        commands: &mut EditorCommands,
        radio_app_state: RadioAppState,
    ) {
        // Register Commands
        commands.register(SplitPanelCommand(radio_app_state));
        commands.register(ToggleCommanderCommand(radio_app_state));
        commands.register(OpenSettingsCommand(radio_app_state));

        // Register Shortcuts
        keyboard_shorcuts.register(
            |data: &KeyboardData,
             commands: &mut EditorCommands,
             mut radio_app_state: RadioAppState| {
                let is_pressing_alt = data.modifiers == Modifiers::ALT;

                match data.code {
                    // Pressing `Esc`
                    Code::Escape => {
                        commands.trigger(ToggleCommanderCommand::id());
                    }
                    // Pressing `Alt E`
                    Code::KeyE if is_pressing_alt => {
                        let mut app_state = radio_app_state.write_channel(Channel::Global);
                        if *app_state.focused_view() == EditorView::FilesExplorer {
                            app_state.set_focused_view(EditorView::Panels)
                        } else {
                            app_state.set_focused_view(EditorView::FilesExplorer)
                        }
                    }

                    _ => return false,
                }
                true
            },
        );
    }
}

#[derive(Clone)]
pub struct SplitPanelCommand(pub RadioAppState);

impl SplitPanelCommand {
    pub fn id() -> &'static str {
        "split-panel"
    }
}

impl EditorCommand for SplitPanelCommand {
    fn matches(&self, input: &str) -> bool {
        self.text().to_lowercase().contains(input)
    }

    fn id(&self) -> &str {
        Self::id()
    }

    fn text(&self) -> &str {
        "Split Panel"
    }

    fn run(&self) {
        let mut radio_app_state = self.0;
        let len_panels = radio_app_state.read().panels().len();
        let mut app_state = radio_app_state.write_channel(Channel::Global);
        app_state.push_panel(Panel::new());
        app_state.set_focused_panel(len_panels - 1);
    }
}

#[derive(Clone)]
pub struct ToggleCommanderCommand(pub RadioAppState);

impl ToggleCommanderCommand {
    pub fn id() -> &'static str {
        "toggle-commander"
    }
}

impl EditorCommand for ToggleCommanderCommand {
    fn is_visible(&self) -> bool {
        // It doesn't make sense to show this command in the Commander.
        false
    }

    fn matches(&self, _input: &str) -> bool {
        false
    }

    fn id(&self) -> &str {
        Self::id()
    }

    fn text(&self) -> &str {
        "Toggle Commander"
    }

    fn run(&self) {
        let mut radio_app_state = self.0;
        let mut app_state = radio_app_state.write_channel(Channel::Global);
        if app_state.focused_view == EditorView::Commander {
            app_state.set_focused_view_to_previous();
        } else {
            app_state.set_focused_view(EditorView::Commander);
        }
    }
}

#[derive(Clone)]
pub struct OpenSettingsCommand(pub RadioAppState);

impl OpenSettingsCommand {
    pub fn id() -> &'static str {
        "open-settings"
    }
}

impl EditorCommand for OpenSettingsCommand {
    fn matches(&self, input: &str) -> bool {
        self.text().to_lowercase().contains(&input.to_lowercase())
    }

    fn id(&self) -> &str {
        Self::id()
    }

    fn text(&self) -> &str {
        "Open Settings"
    }

    fn run(&self) {
        let mut radio_app_state = self.0;
        let mut app_state = radio_app_state.write_channel(Channel::Global);
        Settings::open_with(&mut app_state);
    }
}
