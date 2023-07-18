use freya::prelude::*;

#[allow(non_snake_case)]
#[inline_props]
pub fn Tab<'a>(
    cx: Scope<'a>,
    value: &'a str,
    onclick: EventHandler<(), 'a>,
    onclickclose: EventHandler<(), 'a>,
    is_selected: bool,
) -> Element {
    let status = use_state(cx, ButtonStatus::default);
    let theme = use_get_theme(cx);

    let onmouseenter = move |_| {
        status.set(ButtonStatus::Hovering);
    };

    let onmouseleave = move |_| {
        status.set(ButtonStatus::default());
    };

    let (background, shadow) = match *status.get() {
        _ if *is_selected => ("rgb(35, 35, 35)", "0 2 17 2 rgb(0, 0, 0, 100)"),
        ButtonStatus::Hovering => ("rgb(30, 30, 30)", "none"),
        ButtonStatus::Idle => ("transparent", "transparent"),
    };
    let color = theme.button.font_theme.color;

    render!(
        rect {
            padding: "2",
            width: "150",
            height: "100%",
            rect {
                display: "center",
                width: "100%",
                height: "100%",
                rect {
                    color: "{color}",
                    background: "{background}",
                    shadow: "{shadow}",
                    corner_radius: "5",
                    onclick: move |_| onclick.call(()),
                    onmouseenter: onmouseenter,
                    onmouseleave: onmouseleave,
                    padding: "10 12",
                    width: "100%",
                    direction: "horizontal",
                    label {
                        width: "calc(100% - 25)",
                        max_lines: "1",
                        text_overflow: "ellipsis",
                        font_size: "15",
                        "{value}"
                    }
                    rect {
                        width: "20",
                        height: "20",
                        onclick: move |_| onclickclose.call(()),
                        display: "center",
                        direction: "both",
                        corner_radius: "100",
                        padding: "4",
                        background: "{background}",
                        label {
                            font_size: "13",
                            "X"
                        }
                    }
                }
            }
        }
    )
}
