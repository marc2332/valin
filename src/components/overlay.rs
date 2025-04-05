use crate::state::Channel;
use dioxus_radio::prelude::use_radio;
use freya::prelude::*;

#[component]
pub fn Overlay(children: Element) -> Element {
    let mut radio_app_state = use_radio(Channel::Global);

    let onglobalmousedown = move |_| {
        let mut app_state = radio_app_state.write_channel(Channel::Global);
        app_state.set_focused_view_to_previous();
    };

    rsx!(
        rect {
            width: "100%",
            height: "0",
            layer: "-9999",
            onglobalmousedown,
            rect {
                width: "100%",
                height: "100v",
                main_align: "center",
                cross_align: "center",
                rect {
                    background: "rgb(35, 38, 39)",
                    shadow: "0 4 15 8 rgb(0, 0, 0, 0.3)",
                    corner_radius: "12",
                    onmousedown: |e| {
                        e.stop_propagation();
                    },
                    width: "500",
                    padding: "5",
                    {children}
                }
            }
        }
    )
}
