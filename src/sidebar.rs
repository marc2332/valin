use freya::prelude::*;

use crate::panels::{PanelTab, PanelsManager};

#[inline_props]
#[allow(non_snake_case)]
pub fn Sidebar(cx: Scope, panels_manager: UseState<PanelsManager>) -> Element {
    let open_settings = move |_| {
        to_owned![panels_manager];
        panels_manager.with_mut(|panels_manager| {
            panels_manager.push_tab(PanelTab::Config, panels_manager.focused_panel(), true);
        });
    };

    render!(
        rect {
            overflow: "clip",
            direction: "vertical",
            width: "60",
            height: "100%",
            Button {
                label {
                    "Files"
                }
            }
            Button {
                onclick: open_settings,
                label {
                    "Conf"
                }
            }
        }
    )
}
