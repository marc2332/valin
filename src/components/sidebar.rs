use dioxus_radio::prelude::use_radio;
use freya::prelude::*;

use crate::state::{AppState, Channel, EditorSidePanel, PanelTab};

#[allow(non_snake_case)]
pub fn EditorSidebar() -> Element {
    let mut radio_app_state = use_radio::<AppState, Channel>(Channel::Global);

    let open_settings = move |_| {
        let app_state = radio_app_state.write();
        //app_state.open_file();
    };

    let toggle_file_explorer = move |_| {
        let mut app_state = radio_app_state.write();
        app_state.toggle_side_panel(EditorSidePanel::FileExplorer);
    };

    rsx!(
        rect {
            overflow: "clip",
            direction: "vertical",
            width: "60",
            height: "100%",
            padding: "1",
            cross_align: "center",
            SideBarButton {
                Button {
                    theme: theme_with!(ButtonTheme {
                        width: "100%".into(),
                        padding: "10 6".into(),
                    }),
                    onclick: toggle_file_explorer,
                    label {
                        width: "100%",
                        "📂"
                    }
                }
            }
            SideBarButton {
                Button {
                    theme: theme_with!(ButtonTheme {
                        width: "100%".into(),
                        padding: "10 6".into(),
                    }),
                    onclick: open_settings,
                    label {
                        width: "100%",
                        "⚙️"
                    }
                }
            }
        }
    )
}

#[derive(Props, Clone, PartialEq)]
struct SideBarButtonProps {
    children: Element,
}

#[allow(non_snake_case)]
fn SideBarButton(props: SideBarButtonProps) -> Element {
    rsx!(
        rect {
            direction: "horizontal",
            main_align: "center",
            {props.children}
        }
    )
}
