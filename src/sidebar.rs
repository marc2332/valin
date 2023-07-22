use freya::prelude::*;

use crate::manager::{EditorManager, PanelTab};

#[inline_props]
#[allow(non_snake_case)]
pub fn Sidebar(cx: Scope, editor_manager: UseState<EditorManager>) -> Element {
    let open_settings = move |_| {
        to_owned![editor_manager];
        editor_manager.with_mut(|editor_manager| {
            editor_manager.push_tab(PanelTab::Config, editor_manager.focused_panel(), true);
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
