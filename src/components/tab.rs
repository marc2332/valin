use freya::prelude::*;
use winit::window::CursorIcon;

#[allow(non_snake_case)]
#[component]
pub fn Tab<'a>(
    cx: Scope<'a>,
    value: &'a str,
    onclick: EventHandler<(), 'a>,
    onclickaction: EventHandler<(), 'a>,
    is_selected: bool,
    is_edited: bool,
) -> Element {
    let status = use_state(cx, ButtonStatus::default);
    let theme = use_get_theme(cx);
    let platform = use_platform(cx);

    use_on_destroy(cx, {
        to_owned![status, platform];
        move || {
            if *status.current() == ButtonStatus::Hovering {
                platform.set_cursor(CursorIcon::default());
            }
        }
    });

    let onmouseenter = {
        to_owned![status, platform];
        move |_| {
            platform.set_cursor(CursorIcon::Hand);
            status.set(ButtonStatus::Hovering);
        }
    };

    let onmouseleave = move |_| {
        platform.set_cursor(CursorIcon::default());
        status.set(ButtonStatus::default());
    };

    let background = match *status.get() {
        _ if *is_selected => "rgb(37, 37, 37)",
        ButtonStatus::Hovering => "rgb(30, 30, 30)",
        ButtonStatus::Idle => "transparent",
    };
    let color = theme.button.font_theme.color;
    let selected_color = if *is_selected {
        "rgb(60, 60, 60)"
    } else {
        background
    };
    let is_hovering = *status.get() == ButtonStatus::Hovering;

    render!(
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
                    font_family: "jetbrains mono",
                    width: "calc(100% - 15)",
                    max_lines: "1",
                    text_overflow: "ellipsis",
                    font_size: "14",
                    "{value}"
                }
                rect {
                    width: "15",
                    height: "20",
                    onclick: move |_| onclickaction.call(()),
                    main_align: "center",
                    cross_align: "center",
                    corner_radius: "100",
                    padding: "4",
                    background: "{background}",
                    if *is_edited {
                        rsx!(
                            rect {
                                background: "white",
                                width: "10",
                                height: "10",
                                corner_radius: "100",
                            }
                        )
                    } else if is_hovering || *is_selected {
                        rsx!(
                            label {
                                font_size: "13",
                                "X"
                            }
                        )
                    }
                }
            }
        }
    )
}
