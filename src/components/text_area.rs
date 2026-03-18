use freya::{
    prelude::*,
    text_edit::{EditableConfig, EditableEvent, EditorLine, TextEditor, use_editable},
};

/// [`TextArea`] component.
#[derive(PartialEq)]
pub struct TextArea {
    /// Placeholder text for when there is no text.
    pub placeholder: String,
    /// Current value of the TextArea.
    pub value: Writable<String>,
    /// Handler for the `on_change` event.
    pub on_change: Option<EventHandler<String>>,
    /// Handler for the `on_submit` event.
    pub on_submit: Option<EventHandler<String>>,
}

impl TextArea {
    pub fn new(value: impl Into<Writable<String>>) -> Self {
        Self {
            placeholder: String::new(),
            value: value.into(),
            on_change: None,
            on_submit: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn on_change(mut self, on_change: impl Into<EventHandler<String>>) -> Self {
        self.on_change = Some(on_change.into());
        self
    }

    pub fn on_submit(mut self, on_submit: impl Into<EventHandler<String>>) -> Self {
        self.on_submit = Some(on_submit.into());
        self
    }
}

impl Component for TextArea {
    fn render(&self) -> impl IntoElement {
        let mut status = use_state(InputStatus::default);
        let value = self.value.clone();
        let mut editable = use_editable(|| value.read().to_string(), EditableConfig::new);
        let focus = use_focus();
        let holder = use_state(ParagraphHolder::default);

        if &*value.read() != editable.editor().read().rope() {
            editable.editor_mut().write().set(&value.read());
            editable.editor_mut().write().editor_history().clear();
            editable.editor_mut().write().clear_selection();
        }

        let on_change = self.on_change.clone();
        let on_submit = self.on_submit.clone();

        let mut key_value = value.clone();
        let onkeydown = move |e: Event<KeyboardEventData>| {
            if let Key::Named(NamedKey::Enter) = e.key {
                if let Some(on_submit) = &on_submit {
                    on_submit.call(editable.editor().peek().to_string());
                }
            } else {
                editable.process_event(EditableEvent::KeyDown {
                    key: &e.key,
                    modifiers: e.modifiers,
                });
                let text = editable.editor().peek().to_string();
                *key_value.write() = text.clone();
                if let Some(on_change) = &on_change {
                    on_change.call(text);
                }
            }
        };

        let on_mousemove = move |e: Event<MouseEventData>| {
            editable.process_event(EditableEvent::Move {
                location: e.element_location,
                editor_line: freya::text_edit::EditorLine::SingleParagraph,
                holder: &holder.read(),
            });
        };

        let on_mouseenter = move |_: Event<PointerEventData>| {
            Cursor::set(CursorIcon::Text);
            *status.write() = InputStatus::Hovering;
        };

        let on_mouse_leave = move |_: Event<PointerEventData>| {
            Cursor::set(CursorIcon::default());
            *status.write() = InputStatus::default();
        };

        let on_mouse_down = move |e: Event<MouseEventData>| {
            focus.request_focus();
            editable.process_event(EditableEvent::Down {
                location: e.element_location,
                editor_line: freya::text_edit::EditorLine::SingleParagraph,
                holder: &holder.read(),
            });
        };

        let on_global_mouse_up = move |_: Event<PointerEventData>| {
            editable.process_event(EditableEvent::Release);
        };

        let text_selection = editable
            .editor()
            .read()
            .get_visible_selection(EditorLine::SingleParagraph);
        let cursor_char = editable.editor().read().cursor_pos();

        let placeholder = self.placeholder.clone();
        let value_str = value.read();
        let (text_color, display_text) = if value_str.is_empty() {
            (Color::from((110, 118, 129)), placeholder)
        } else {
            (Color::from((230, 237, 243)), value_str.clone())
        };

        rect()
            .overflow(Overflow::Clip)
            .width(Size::fill())
            .color(text_color)
            .background((8, 8, 12))
            .corner_radius(8.)
            .border(Border::new().width(1.).fill((33, 38, 45)))
            .padding((8., 6.))
            .margin((0., 0., 2., 0.))
            .a11y_id(focus.a11y_id())
            .a11y_role(AccessibilityRole::TextInput)
            .a11y_auto_focus(true)
            .on_key_down(onkeydown)
            .child(
                paragraph()
                    .holder(holder.read().clone())
                    .margin((6., 10.))
                    .on_pointer_enter(on_mouseenter)
                    .on_pointer_leave(on_mouse_leave)
                    .on_mouse_move(on_mousemove)
                    .on_mouse_down(on_mouse_down)
                    .on_global_pointer_press(on_global_mouse_up)
                    .width(Size::fill())
                    .cursor_index(cursor_char)
                    .cursor_color(text_color)
                    .max_lines(1)
                    .highlights(text_selection.map(|h| vec![h]))
                    .span(Span::new(display_text)),
            )
    }
}
