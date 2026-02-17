use freya::prelude::KeyboardEventData;

use super::{EditorCommands, RadioAppState};

type KeyboardShortcutHandler =
    dyn Fn(&KeyboardEventData, &mut EditorCommands, RadioAppState) -> bool;

#[derive(Default)]
pub struct KeyboardShortcuts {
    handlers: Vec<Box<KeyboardShortcutHandler>>,
}

impl KeyboardShortcuts {
    pub fn register(
        &mut self,
        handler: impl Fn(&KeyboardEventData, &mut EditorCommands, RadioAppState) -> bool + 'static,
    ) {
        self.handlers.push(Box::new(handler))
    }

    pub fn run(
        &self,
        data: &KeyboardEventData,
        editor_commands: &mut EditorCommands,
        radio_app_state: RadioAppState,
    ) {
        for event_handler in &self.handlers {
            let res = (event_handler)(data, editor_commands, radio_app_state);

            if res {
                break;
            }
        }
    }
}
