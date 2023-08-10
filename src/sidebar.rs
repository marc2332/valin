use freya::prelude::*;

use crate::manager::{EditorManagerWrapper, PanelTab};

#[inline_props]
#[allow(non_snake_case)]
pub fn Sidebar(cx: Scope, editor_manager: EditorManagerWrapper) -> Element {
    let open_settings = move |_| {
        let focused_panel = editor_manager.current().focused_panel();
        editor_manager
            .write(None)
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
