use freya::prelude::*;

#[allow(non_snake_case)]
#[inline_props]
pub fn FileTab<'a>(
    cx: Scope<'a>,
    value: &'a str,
    onclick: EventHandler<(), 'a>,
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
        _ if *is_selected => theme.button.hover_background,
        ButtonStatus::Hovering => theme.button.hover_background,
        ButtonStatus::Idle => theme.button.background,
    };
    let color = theme.button.font_theme.color;

    render!(
        rect {
            padding: "2",
            width: "150",
            height: "100%",
            rect {
                color: "{color}",
                background: "{background}",
                shadow: "0 2 17 2 rgb(0, 0, 0, 100)",
                radius: "5",
                onclick: move |_| onclick.call(()),
                onmouseenter: onmouseenter,
                onmouseleave: onmouseleave,
                padding: "7",
                width: "100%",
                height: "100%",
                display: "center",
                direction: "both",
                label {
                    "{value}"
                }
            }
        }
    )
}
