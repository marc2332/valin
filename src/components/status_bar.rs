use dioxus_radio::prelude::use_radio;
use freya::prelude::*;

use crate::{
    state::{Channel, EditorSidePanel, EditorView},
    tabs::{editor::TabEditorUtils, settings::Settings},
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

    let open_settings = move |_| {
        let mut app_state = radio_app_state.write_channel(Channel::Global);
        Settings::open_with(&mut app_state);
    };

    let toggle_file_explorer = move |_| {
        let mut app_state = radio_app_state.write_channel(Channel::Global);
        app_state.toggle_side_panel(EditorSidePanel::FileExplorer);
    };

    let app_state = radio_app_state.read();
    let panel = app_state.panel(app_state.focused_panel);
    let tab_data = {
        if let Some(active_tab) = panel.active_tab() {
            panel
                .tab(active_tab)
                .as_text_editor()
                .map(|editor_tab| (editor_tab.editor.cursor(), editor_tab.editor.editor_type()))
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
            rect {
                width: "50%",
                direction: "horizontal",
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
                for (name, msg) in props.lsp_statuses.read().iter() {
                    StatusBarItem {
                        label {
                            "{name} {msg}"
                        }
                    }
                }
            }
            rect {
                width: "50%",
                direction: "horizontal",
                main_align: "end",
                if let Some((cursor, editor_type)) = tab_data {
                    StatusBarItem {
                        label {
                            "Ln {cursor.row() + 1}, Col {cursor.col() + 1}"
                        }
                    }
                    StatusBarItem {
                        label {
                            "{editor_type.language_id()}"
                        }
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
