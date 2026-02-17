use freya::prelude::*;

use crate::state::{AppState, PanelTab, PanelTabData, TabId, TabProps};

pub struct WelcomeTab {
    id: TabId,
    focus_id: AccessibilityId,
}

impl PanelTab for WelcomeTab {
    fn get_data(&self) -> PanelTabData {
        PanelTabData {
            id: self.id,
            title: "welcome".to_string(),
            edited: false,
            focus_id: self.focus_id,
            content_id: "welcome".to_string(),
        }
    }
    fn render(&self) -> fn(&TabProps) -> Element {
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
            id: TabId::new(),
            focus_id: Focus::new_id(),
        }
    }

    pub fn open_with(app_state: &mut AppState) {
        app_state.push_tab(Self::new(), app_state.focused_panel);
    }
}

pub fn render(_: &TabProps) -> Element {
    rect()
        .padding((32., 8.))
        .expanded()
        .cross_align(Alignment::center())
        .child(
            MarkdownViewer::new(
                "
**Valin** ⚒️ is a **Work-In-Progress** cross-platform code editor, made with Freya 🦀 and Rust.

> **Valin** name is derived from Dvalinn and it was previously known as `freya-editor`.

",
            )
            .width(Size::percent(70.)),
        )
        .into()
    // rsx!(
    //     rect {
    //         height: "100%",
    //         width: "100%",
    //         background: "rgb(29, 32, 33)",
    //         padding: "20",
    //         Link {
    //             to: "https://github.com/marc2332/freya",
    //             tooltip: LinkTooltip::None,
    //             label {
    //                 "freya source code"
    //             }
    //         }
    //         Link {
    //             to: "https://github.com/marc2332/valin",
    //             tooltip: LinkTooltip::None,
    //             label {
    //                 "Valin source code"
    //             }
    //         }
    //     }
    // )
}
