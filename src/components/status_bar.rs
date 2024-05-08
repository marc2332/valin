use dioxus_radio::prelude::use_radio;
use dioxus_sdk::clipboard::use_clipboard;
use freya::prelude::*;

use crate::{
    state::{AppState, Channel, EditorSidePanel, EditorView},
    LspStatuses,
};

#[derive(Props, Clone, PartialEq)]
pub struct StatusBarProps {
    lsp_statuses: LspStatuses,
    focused_view: EditorView,
}

#[allow(non_snake_case)]
pub fn StatusBar(props: StatusBarProps) -> Element {
    let mut radio_app_state = use_radio(Channel::ActiveTab);
    let clipboard = use_clipboard();

    let open_settings = move |_| {
        let mut app_state = radio_app_state.write_channel(Channel::Global);
        app_state.open_settings(clipboard);
    };

    let toggle_file_explorer = move |_| {
        let mut app_state = radio_app_state.write_channel(Channel::Global);
        app_state.toggle_side_panel(EditorSidePanel::FileExplorer);
    };

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
            padding: "0 2",
            color: "rgb(220, 220, 220)",
            StatusBarItem {
                onclick: toggle_file_explorer,
                label {
                    "📁"
                }
            }
            StatusBarItem {
                onclick: open_settings,
                label {
                    "⚙️"
                }
            }
            StatusBarItem {
                label {
                    "{props.focused_view}"
                }
            }
            if let Some(cursor) = cursor {
                StatusBarItem {
                    label {
                        "Ln {cursor.row() + 1}, Col {cursor.col() + 1}"
                    }
                }
            }
            for (name, msg) in props.lsp_statuses.read().iter() {
                StatusBarItem {
                    label {
                        "{name} {msg}"
                    }
                }
            }
        }
    )
}

#[allow(non_snake_case)]
#[component]
fn StatusBarItem(children: Element, onclick: Option<EventHandler<()>>) -> Element {
    rsx!(
        Button {
            onclick: move |_| {
                if let Some(onclick) = onclick {
                    onclick.call(());
                }
            },
            theme: theme_with!(ButtonTheme {
                margin: "2".into(),
                padding: "4 6".into(),
                background: "none".into(),
                border_fill: "none".into(),
            }),
            {children}
        }
    )
}
