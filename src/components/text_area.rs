use freya::{
    prelude::*,
    text_edit::{EditableConfig, EditableEvent, EditorLine, TextEditor, use_editable},
};

/// [`TextArea`] component.
#[derive(PartialEq)]
pub struct TextArea {
    /// Placeholder text for when there is no text.
    pub placeholder: String,
    /// Current value of the TextArea
    pub value: String, // TODO: CHANGE
    /// Handler for the `onchange` event.
    pub onchange: EventHandler<String>,
    /// Handler for the `on_submit` event.
    pub on_submit: EventHandler<String>,
}

impl TextArea {
    pub fn new() -> Self {
        Self {
            placeholder: String::new(),
            value: String::new(),
            onchange: EventHandler::new(|_| {}),
            on_submit: EventHandler::new(|_| {}),
        }
    }
    // TODO: CHANGE
    pub fn placeholder<P: Into<String>>(mut self, placeholder: P) -> Self {
        self.placeholder = placeholder.into();
        self
    }
    // TODO: CHANGE
    pub fn value<V: Into<String>>(mut self, value: V) -> Self {
        self.value = value.into();
        self
    }
    // TODO: CHANGE
    pub fn onchange<F>(mut self, onchange: F) -> Self
    where
        F: FnMut(String) + 'static,
    {
        self.onchange = EventHandler::new(onchange);
        self
    }
    // TODO: CHANGE
    pub fn on_submit<F>(mut self, on_submit: F) -> Self
    where
        F: FnMut(String) + 'static,
    {
        self.on_submit = EventHandler::new(on_submit);
        self
    }
}

impl Component for TextArea {
    fn render(&self) -> impl IntoElement {
        let mut status = use_state(InputStatus::default);
        let value = self.value.clone();
        let mut editable = use_editable(|| value.clone(), EditableConfig::new);
        let focus = use_focus();
        let holder = use_state(ParagraphHolder::default);

        if value != *editable.editor().read().rope() {
            editable.editor_mut().write().set(&value);
            editable.editor_mut().write().editor_history().clear();
            editable.editor_mut().write().clear_selection();
        }

        let onchange = self.onchange.clone();
        let on_submit = self.on_submit.clone();

        let onkeydown = move |e: Event<KeyboardEventData>| {
            if let Key::Named(NamedKey::Enter) = e.key {
                on_submit.call(editable.editor().peek().to_string());
            } else {
                editable.process_event(EditableEvent::KeyDown {
                    key: &e.key,
                    modifiers: e.modifiers,
                });
                onchange.call(editable.editor().peek().to_string());
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

        let on_global_pointer_press = move |_: Event<PointerEventData>| {
            editable.process_event(EditableEvent::Release);
        };

        let text_selection = editable
            .editor()
            .read()
            .get_visible_selection(EditorLine::SingleParagraph);
        let cursor_char = editable.editor().read().cursor_pos();

        let placeholder = self.placeholder.clone();
        let (text_color, display_text) = if self.value.is_empty() {
            (Color::from((210, 210, 210)), placeholder)
        } else {
            (Color::WHITE, self.value.clone())
        };

        rect()
            .overflow(Overflow::Clip)
            .width(Size::fill())
            .color(text_color)
            .background((45, 48, 49))
            .corner_radius(8.)
            .border(Border::new().width(1.).fill((60, 63, 64)))
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
                    .on_global_pointer_press(on_global_pointer_press)
                    .width(Size::fill())
                    .cursor_index(cursor_char)
                    .cursor_color(text_color)
                    .max_lines(1)
                    .highlights(text_selection.map(|h| vec![h]))
                    .span(Span::new(display_text)),
            )
    }
}
