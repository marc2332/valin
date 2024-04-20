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
    let platform = use_platform();
    let mut status = use_signal(InputStatus::default);
    let mut editable = use_editable(
        || EditableConfig::new(props.value.to_string()),
        EditableMode::MultipleLinesSingleEditor,
    );
    let mut focus = use_focus();

    use_hook(|| {
        focus.focus();
    });

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

    let onmouseover = move |e: MouseEvent| {
        editable.process_event(&EditableEvent::MouseOver(e.data, 0));
    };

    let onmouseenter = move |_| {
        platform.set_cursor(CursorIcon::Text);
        *status.write() = InputStatus::Hovering;
    };

    let onmouseleave = move |_| {
        platform.set_cursor(CursorIcon::default());
        *status.write() = InputStatus::default();
    };

    let color = &button_theme.font_theme.color;
    let focus_id = focus.attribute();
    let cursor_reference = editable.cursor_attr();
    let highlights = editable.highlights_attr(0);
    let cursor_char = editable.editor().read().cursor_pos().to_string();

    rsx!(
        rect {
            overflow: "clip",
            width: "100%",
            color: "{color}",
            corner_radius: "5",
            padding: "12 10",
            cursor_reference,
            focus_id,
            focusable: "true",
            role: "textInput",
            paragraph {
                margin: "8 12",
                onkeydown,
                onmouseenter,
                onmouseleave,
                onmouseover,
                width: "100%",
                cursor_id: "0",
                cursor_index: "{cursor_char}",
                cursor_mode: "editable",
                cursor_color: "{color}",
                max_lines: "1",
                highlights,
                text {
                    "{props.value}"
                }
            }
        }
    )
}
