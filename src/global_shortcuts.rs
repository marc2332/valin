#[allow(non_snake_case)]
pub mod GlobalShortcuts {
    use freya::events::{Key, KeyboardData};

    use crate::state::{Channel, EditorView, KeyboardShortcuts, RadioAppState};

    pub fn register_handlers(keyboard_shorcuts: &mut KeyboardShortcuts) {
        keyboard_shorcuts.register(|data: &KeyboardData, mut radio_app_state: RadioAppState| {
            if Key::Escape == data.key {
                let mut app_state = radio_app_state.write_channel(Channel::Global);
                if app_state.focused_view == EditorView::Commander {
                    app_state.set_focused_view_to_previous();
                } else {
                    app_state.set_focused_view(EditorView::Commander);
                }

                false
            } else {
                true
            }
        });
    }
}
