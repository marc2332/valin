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

    let background = match *status.get() {
        _ if *is_selected => "rgb(37, 37, 37)",
        ButtonStatus::Hovering => "rgb(30, 30, 30)",
        ButtonStatus::Idle => "transparent",
    };
    let color = theme.button.font_theme.color;

    render!(
        rect {
            margin: "0 2",
            corner_radius: "5",
            color: "{color}",
            background: "{background}",
            onclick: move |_| onclick.call(()),
            onmouseenter: onmouseenter,
            onmouseleave: onmouseleave,
            padding: "8 12",
            height: "100%",
            width: "130",
            display: "center",
            rect {
                height: "100%",
                width: "100%",
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
    )
}
