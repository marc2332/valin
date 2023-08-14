use freya::prelude::*;

use crate::manager::{use_manager, PanelTab};

#[allow(non_snake_case)]
pub fn Sidebar(cx: Scope) -> Element {
    let manager = use_manager(cx);

    let open_settings = move |_| {
        let focused_panel = manager.current().focused_panel();
        manager
            .global_write()
            .push_tab(PanelTab::Config, focused_panel, true);
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
