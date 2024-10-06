use freya::prelude::{keyboard::Key, *};

/// [`TextArea`] component properties.
#[derive(Props, Clone, PartialEq)]
pub struct TextAreaProps {
    /// Placerholder text for when there is no text.
    pub placeholder: &'static str,
    /// Current value of the TextArea
    pub value: String,
    /// Handler for the `onchange` event.
    pub onchange: EventHandler<String>,
    /// Handler for the `onsubmit` event.
    pub onsubmit: EventHandler<String>,
}

#[allow(non_snake_case)]
pub fn TextArea(props: TextAreaProps) -> Element {
    let theme = use_applied_theme!(&None, input);
    let platform = use_platform();
    let mut status = use_signal(InputStatus::default);
    let mut editable = use_editable(
        || EditableConfig::new(props.value.to_string()),
        EditableMode::MultipleLinesSingleEditor,
    );
    let focus = use_focus();

    if &props.value != editable.editor().read().rope() {
        editable.editor_mut().write().set(&props.value);
    }

    let onkeydown = move |e: Event<KeyboardData>| {
        if focus.is_focused() {
            if let Key::Enter = e.data.key {
                props.onsubmit.call(editable.editor().peek().to_string());
            } else {
                editable.process_event(&EditableEvent::KeyDown(e.data));
                props.onchange.call(editable.editor().peek().to_string());
            }
        }
    };

    let onmousemove = move |e: MouseEvent| {
        editable.process_event(&EditableEvent::MouseMove(e.data, 0));
    };

    let onmouseenter = move |_| {
        platform.set_cursor(CursorIcon::Text);
        *status.write() = InputStatus::Hovering;
    };

    let onmouseleave = move |_| {
        platform.set_cursor(CursorIcon::default());
        *status.write() = InputStatus::default();
    };

    let cursor_reference = editable.cursor_attr();
    let highlights = editable.highlights_attr(0);
    let cursor_char = editable.editor().read().cursor_pos().to_string();

    let InputTheme {
        border_fill,
        corner_radius,
        background,
        font_theme: FontTheme { color },
        ..
    } = theme;

    let (color, text) = if props.value.is_empty() {
        ("rgb(210, 210, 210)", props.placeholder)
    } else {
        (color.as_ref(), props.value.as_str())
    };

    rsx!(
        rect {
            overflow: "clip",
            width: "100%",
            color: "{color}",
            background: "{background}",
            corner_radius: "{corner_radius}",
            border: "1 solid {border_fill}",
            padding: "8 6",
            margin: "0 0 2 0",
            cursor_reference,
            a11y_id: focus.attribute(),
            a11y_role: "text-input",
            a11y_auto_focus: "true",
            onkeydown,
            paragraph {
                margin: "6 10",
                onmouseenter,
                onmouseleave,
                onmousemove,
                width: "100%",
                cursor_id: "0",
                cursor_index: "{cursor_char}",
                cursor_mode: "editable",
                cursor_color: "{color}",
                max_lines: "1",
                highlights,
                text {
                    "{text}"
                }
            }
        }
    )
}
