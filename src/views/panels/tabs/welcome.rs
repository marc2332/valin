use freya::prelude::*;

use crate::state::{AppState, PanelTab, PanelTabData, TabProps};

pub struct WelcomeTab {
    focus_id: AccessibilityId,
}

impl PanelTab for WelcomeTab {
    fn get_data(&self) -> PanelTabData {
        PanelTabData {
            id: "welcome".to_string(),
            title: "welcome".to_string(),
            edited: false,
            focus_id: self.focus_id,
        }
    }
    fn render(&self) -> fn(TabProps) -> Element {
        render
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl WelcomeTab {
    pub fn new() -> Self {
        Self {
            focus_id: UseFocus::new_id(),
        }
    }

    pub fn open_with(app_state: &mut AppState) {
        app_state.push_tab(Self::new(), app_state.focused_panel, true);
    }
}

pub fn render(_: TabProps) -> Element {
    rsx!(
        rect {
            height: "100%",
            width: "100%",
            background: "rgb(35, 35, 35)",
            padding: "20",
            Link {
                to: "https://github.com/marc2332/freya",
                tooltip: LinkTooltip::None,
                label {
                    "freya source code"
                }
            }
            Link {
                to: "https://github.com/marc2332/valin",
                tooltip: LinkTooltip::None,
                label {
                    "Valin source code"
                }
            }
        }
    )
}
