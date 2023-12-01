use freya::prelude::*;
use winit::window::CursorIcon;

#[allow(non_snake_case)]
#[inline_props]
pub fn Tab<'a>(
    cx: Scope<'a>,
    value: &'a str,
    onclick: EventHandler<(), 'a>,
    onclickaction: EventHandler<(), 'a>,
    is_selected: bool,
    is_edited: bool
) -> Element {
    let status = use_state(cx, ButtonStatus::default);
    let theme = use_get_theme(cx);
    let platform = use_platform(cx);

    use_on_unmount(cx, {
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
    let border = if *is_selected {
        "rgb(60, 60, 60)"
    } else {
        "transparent"
    };
    let is_hovering =  *status.get() == ButtonStatus::Hovering;

    render!(
        rect {
            margin: "0 2",
            corner_radius: "5",
            color: "{color}",
            background: "{background}",
            onclick: move |_| onclick.call(()),
            onmouseenter: onmouseenter,
            onmouseleave: onmouseleave,
            padding: "0 12",
            height: "100%",
            width: "130",
            main_align: "center",
            cross_align: "center",
            direction: "horizontal",
            border: "2 solid {border}",
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
                            width: "15",
                            height: "15",
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
    )
}
