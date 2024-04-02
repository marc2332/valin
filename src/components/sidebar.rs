use dioxus_radio::prelude::use_radio;
use freya::prelude::*;

use crate::editor_manager::{Channel, EditorManager, PanelTab};

#[allow(non_snake_case)]
pub fn EditorSidebar() -> Element {
    let mut radio = use_radio::<EditorManager, Channel>(Channel::All);

    let open_settings = move |_| {
        let focused_panel = radio.read().focused_panel();
        radio
            .write_channel(Channel::All)
            .push_tab(PanelTab::Config, focused_panel, true);
    };

    rsx!(
        rect {
            overflow: "clip",
            direction: "vertical",
            width: "60",
            height: "100%",
            padding: "2",
            cross_align: "center",
            SideBarButton {
                Button {
                    theme: theme_with!(ButtonTheme {
                        width: "100%".into(),
                        padding: "10 8".into(),
                    }),
                    label {
                        "Files"
                    }
                }
            }
            SideBarButton {
                Button {
                    theme: theme_with!(ButtonTheme {
                        width: "100%".into(),
                        padding: "10 8".into(),
                    }),
                    onclick: open_settings,
                    label {
                        "Conf"
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
