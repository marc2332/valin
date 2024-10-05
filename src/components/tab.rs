use freya::prelude::*;
use winit::window::CursorIcon;

#[allow(non_snake_case)]
#[component]
pub fn EditorTab(
    value: String,
    onclick: EventHandler<()>,
    onclickaction: EventHandler<()>,
    is_selected: bool,
    is_edited: bool,
) -> Element {
    let mut status = use_signal(ButtonStatus::default);
    let theme = use_applied_theme!(None, button);
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
    let color = theme.font_theme.color;
    let selected_color = if is_selected {
        "rgb(60, 60, 60)"
    } else {
        background
    };
    let is_hovering = *status.read() == ButtonStatus::Hovering;

    rsx!(
        rect {
            width: "140",
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
                onmouseenter,
                onmouseleave,
                height: "fill",
                width: "fill",
                cross_align: "center",
                direction: "horizontal",
                padding: "0 0 0 10",
                label {
                    width: "calc(100% - 24)",
                    max_lines: "1",
                    text_overflow: "ellipsis",
                    text_align: "center",
                    "{value}"
                }
                rect {
                    width: "24",
                    onclick: move |e| {
                        e.stop_propagation();
                        onclickaction.call(());
                    },
                    if is_edited {
                        IndicatorButton {
                            rect {
                                background: "rgb(180, 180, 180)",
                                width: "10",
                                height: "10",
                                corner_radius: "100",
                            }
                        }
                    } else if is_hovering || is_selected {
                        IndicatorButton {
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

#[allow(non_snake_case)]
#[component]
fn IndicatorButton(children: Element) -> Element {
    rsx!(Button {
        theme: theme_with!(ButtonTheme {
            margin: "0".into(),
            corner_radius: "999".into(),
            shadow: "none".into(),
            border_fill: "none".into(),
            width: "20".into(),
            height: "20".into(),
            padding: "0".into(),
        }),
        children
    })
}
