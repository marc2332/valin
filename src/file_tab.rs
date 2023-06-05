use freya::prelude::*;

#[allow(non_snake_case)]
#[inline_props]
pub fn FileTab<'a>(
    cx: Scope<'a>,
    value: &'a str,
    onclick: EventHandler<(), 'a>,
    is_selected: bool,
) -> Element {
    let theme = use_get_theme(cx);
    let button_theme = &theme.button;

    let background = use_state(cx, || <&str>::clone(&button_theme.background));
    let set_background = background.setter();

    use_effect(cx, &button_theme.clone(), move |button_theme| async move {
        set_background(button_theme.background);
    });

    let selected_background = if *is_selected {
        button_theme.hover_background
    } else {
        background.get()
    };

    render!(
        rect {
            padding: "2",
            width: "150",
            height: "100%",
            rect {
                color: "{button_theme.font_theme.color}",
                background: "{selected_background}",
                shadow: "0 5 15 10 black",
                radius: "5",
                onclick: move |_| onclick.call(()),
                onmouseover: move |_| {
                    background.set(theme.button.hover_background);
                },
                onmouseleave: move |_| {
                    background.set(theme.button.background);
                },
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
