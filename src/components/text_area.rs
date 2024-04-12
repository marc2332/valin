use freya::prelude::{keyboard::Key, *};

/// [`TextArea`] component properties.
#[derive(Props, Clone, PartialEq)]
pub struct TextAreaProps {
    /// Current value of the TextArea
    pub value: String,
    /// Handler for the `onchange` event.
    pub onchange: EventHandler<String>,
    /// Handler for the `onsubmit` event.
    pub onsubmit: EventHandler<String>,
}

#[allow(non_snake_case)]
pub fn TextArea(props: TextAreaProps) -> Element {
    let theme = use_get_theme();
    let button_theme = &theme.button;

    let onkeydown = {
        let value = props.value.clone();
        move |e: Event<KeyboardData>| {
            if let Key::Character(text_char) = &e.data.key {
                // Add a new char
                props.onchange.call(format!("{value}{text_char}"));
            } else if let Key::Backspace = e.data.key {
                // Remove the last character
                let mut content = value.to_string();
                content.pop();
                props.onchange.call(content);
            } else if let Key::Enter = e.data.key {
                props.onsubmit.call(value.to_string());
            }
        }
    };

    rsx!(
        rect {
            overflow: "clip",
            onkeydown: onkeydown,
            width: "100%",
            color: "{button_theme.font_theme.color}",
            corner_radius: "5",
            padding: "12 10",
            label {
                "{props.value}"
            }
        }
    )
}
