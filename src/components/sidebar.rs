use freya::prelude::*;

use crate::hooks::{use_manager, PanelTab};

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
            padding: "2",
            cross_align: "center",
            SideBarButton {
                Button {
                    width: "100%",
                    padding: "10 8",
                    label {
                        "Files"
                    }
                }
            }
            SideBarButton {
                Button {
                    width: "100%",
                    padding: "10 8",
                    onclick: open_settings,
                    label {
                        "Conf"
                    }
                }
            }
        }
    )
}

#[derive(Props)]
struct SideBarButtonProps<'a> {
    children: Element<'a>,
}

#[allow(non_snake_case)]
fn SideBarButton<'a>(cx: Scope<'a, SideBarButtonProps<'a>>) -> Element<'a> {
    render!(
        rect {
            direction: "horizontal",
            main_align: "center",
            &cx.props.children
        }
    )
}
