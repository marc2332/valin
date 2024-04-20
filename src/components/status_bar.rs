use dioxus_radio::prelude::use_radio;
use freya::prelude::*;

use crate::{
    state::{Channel, EditorView},
    LspStatuses,
};

#[derive(Props, Clone, PartialEq)]
pub struct StatusBarProps {
    lsp_statuses: LspStatuses,
    focused_view: EditorView,
}

#[allow(non_snake_case)]
pub fn StatusBar(props: StatusBarProps) -> Element {
    let radio_app_state = use_radio(Channel::ActiveTab);

    let cursor = {
        let app_state = radio_app_state.read();
        let panel = app_state.panel(app_state.focused_panel);
        if let Some(active_tab) = panel.active_tab() {
            panel
                .tab(active_tab)
                .as_text_editor()
                .map(|editor| editor.cursor())
        } else {
            None
        }
    };
    rsx!(
        rect {
            width: "100%",
            height: "fill",
            background: "rgb(20, 20, 20)",
            direction: "horizontal",
            cross_align: "center",
            padding: "0 6",
            color: "rgb(220, 220, 220)",
            label {
                "{props.focused_view}"
            }
            if let Some(cursor) = cursor {
                label {
                    " | Ln {cursor.row() + 1}, Col {cursor.col() + 1}"
                }
            }
            for (name, msg) in props.lsp_statuses.read().iter() {
                label {
                    " | {name} {msg}"
                }
            }
        }
    )
}
