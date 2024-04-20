use freya::prelude::*;
use winit::window::CursorIcon;

#[allow(non_snake_case)]
#[component]
pub fn Tab(
    value: String,
    onclick: EventHandler<()>,
    onclickaction: EventHandler<()>,
    is_selected: bool,
    is_edited: bool,
) -> Element {
    let mut status = use_signal(ButtonStatus::default);
    let theme = use_get_theme();
    let platform = use_platform();

    use_drop(move || {
        if *status.read() == ButtonStatus::Hovering {
            platform.set_cursor(CursorIcon::default());
        }
    });

    let onmouseenter = move |_| {
        platform.set_cursor(CursorIcon::Pointer);
        status.set(ButtonStatus::Hovering);
    };

    let onmouseleave = move |_| {
        platform.set_cursor(CursorIcon::default());
        status.set(ButtonStatus::default());
    };

    let background = match *status.read() {
        _ if is_selected => "rgb(37, 37, 37)",
        ButtonStatus::Hovering => "rgb(30, 30, 30)",
        ButtonStatus::Idle => "transparent",
    };
    let color = theme.button.font_theme.color;
    let selected_color = if is_selected {
        "rgb(60, 60, 60)"
    } else {
        background
    };
    let is_hovering = *status.read() == ButtonStatus::Hovering;

    rsx!(
        rect {
            width: "130",
            height: "100%",
            rect {
                height: "2",
                width: "100%",
                background: "{selected_color}"
            }
            rect {
                color: "{color}",
                background: "{background}",
                onclick: move |_| onclick.call(()),
                onmouseenter: onmouseenter,
                onmouseleave: onmouseleave,
                padding: "0 12",
                height: "fill",
                width: "130",
                main_align: "center",
                cross_align: "center",
                direction: "horizontal",
                label {
                    width: "calc(100% - 16)",
                    max_lines: "1",
                    text_overflow: "ellipsis",
                    "{value}"
                }
                rect {
                    width: "16",
                    onclick: move |e| {
                        e.stop_propagation();
                        onclickaction.call(());
                    },
                    if is_edited {
                        rect {
                            padding: "6",
                            rect {
                                background: "rgb(180, 180, 180)",
                                width: "10",
                                height: "10",
                                corner_radius: "100",
                            }
                        }
                    } else if is_hovering || is_selected {
                        Button {
                            theme: theme_with!(ButtonTheme {
                                padding: "6".into(),
                                margin: "0".into(),
                                corner_radius: "999".into(),
                                shadow: "none".into(),
                                border_fill: "none".into(),
                            }),
                            CrossIcon {
                                fill: "rgb(150, 150, 150)",
                            }
                        }
                    }
                }
            }
        }
    )
}
