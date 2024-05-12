use crate::state::{Channel, EditorCommand, EditorView, Panel, RadioAppState};

#[allow(non_snake_case)]
pub mod GlobalDefaults {
    use freya::events::{Code, KeyboardData, Modifiers};

    use crate::state::{Channel, EditorCommands, EditorView, KeyboardShortcuts, RadioAppState};

    use super::{SplitPanelCommand, ToggleCommander, ToggleFocus};

    pub fn init(
        keyboard_shorcuts: &mut KeyboardShortcuts,
        commands: &mut EditorCommands,
        radio_app_state: RadioAppState,
    ) {
        // Register Commands
        commands.register(SplitPanelCommand(radio_app_state));
        commands.register(ToggleCommander(radio_app_state));
        commands.register(ToggleFocus(radio_app_state));

        // Register Shortcuts
        keyboard_shorcuts.register(
            |data: &KeyboardData,
             commands: &mut EditorCommands,
             mut radio_app_state: RadioAppState| {
                let is_pressing_alt = data.modifiers == Modifiers::ALT;

                match data.code {
                    // Pressing `Esc`
                    Code::Escape => {
                        commands.trigger(ToggleCommander::id());
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
pub struct ToggleCommander(pub RadioAppState);

impl ToggleCommander {
    pub fn id() -> &'static str {
        "toggle-commander"
    }
}

impl EditorCommand for ToggleCommander {
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
pub struct ToggleFocus(pub RadioAppState);

impl ToggleFocus {
    pub fn id() -> &'static str {
        "toggle-focus"
    }
}

impl EditorCommand for ToggleFocus {
    fn matches(&self, input: &str) -> bool {
        self.text().to_lowercase().contains(&input.to_lowercase())
    }

    fn id(&self) -> &str {
        Self::id()
    }

    fn text(&self) -> &str {
        "Toggle Focus"
    }

    fn run(&self) {}
}
