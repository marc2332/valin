use freya::prelude::{keyboard::Key, *};

/// [`TextArea`] component properties.
#[derive(Props)]
pub struct TextAreaProps<'a> {
    /// Current value of the TextArea
    pub value: &'a str,
    /// Handler for the `onchange` event.
    pub onchange: EventHandler<'a, String>,
    /// Handler for the `onsubmit` event.
    pub onsubmit: EventHandler<'a, String>,
}

#[allow(non_snake_case)]
pub fn TextArea<'a>(cx: Scope<'a, TextAreaProps<'a>>) -> Element {
    let theme = use_get_theme(cx);
    let button_theme = &theme.button;
    let value = cx.props.value;

    let onkeydown = move |e: Event<KeyboardData>| {
        if let Key::Character(text_char) = &e.data.key {
            // Add a new char
            cx.props.onchange.call(format!("{value}{text_char}"));
        } else if let Key::Backspace = e.data.key {
            // Remove the last character
            let mut content = value.to_string();
            content.pop();
            cx.props.onchange.call(content);
        } else if let Key::Enter = e.data.key {
            cx.props.onsubmit.call(value.to_string());
        }
    };

    render!(
        rect {
            overflow: "clip",
            onkeydown: onkeydown,
            width: "100%",
            height: "100%",
            color: "{button_theme.font_theme.color}",
            corner_radius: "5",
            padding: "12 10",
            label {
                "{value}"
            }
        }
    )
}
