use freya::events::KeyboardData;

use super::RadioAppState;

type KeyboardShortcutHandler = dyn Fn(&KeyboardData, RadioAppState) -> bool;

#[derive(Default)]
pub struct KeyboardShortcuts {
    handlers: Vec<Box<KeyboardShortcutHandler>>,
}

impl KeyboardShortcuts {
    pub fn register(&mut self, handler: impl Fn(&KeyboardData, RadioAppState) -> bool + 'static) {
        self.handlers.push(Box::new(handler))
    }

    pub fn run(&self, data: &KeyboardData, radio_app_state: RadioAppState) {
        for event_handler in &self.handlers {
            let res = (event_handler)(data, radio_app_state);

            if res {
                break;
            }
        }
    }
}
