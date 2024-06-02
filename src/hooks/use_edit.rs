use dioxus::{dioxus_core::AttributeValue, prelude::use_memo};

use crate::tabs::editor::AppStateEditorUtils;
use freya::common::{CursorLayoutResponse, EventMessage, TextGroupMeasurement};
use freya::prelude::{keyboard::Modifiers, *};
use freya_node_state::CursorReference;
use tokio::sync::mpsc::unbounded_channel;
use uuid::Uuid;

use crate::state::RadioAppState;

/// Manage an editable content.
#[derive(Clone, Copy, PartialEq)]
pub struct UseEdit {
    pub(crate) radio: RadioAppState,
    pub(crate) cursor_reference: Memo<CursorReference>,
    pub(crate) dragging: Signal<TextDragging>,
    pub(crate) platform: UsePlatform,
    pub(crate) panel_index: usize,
    pub(crate) tab_index: usize,
}

impl UseEdit {
    /// Create a cursor attribute.
    pub fn cursor_attr(&self) -> AttributeValue {
        AttributeValue::any_value(CustomAttributeValues::CursorReference(
            self.cursor_reference.peek().clone(),
        ))
    }

    /// Check if there is any highlight at all.
    pub fn has_any_highlight(&self) -> bool {
        let app_state = self.radio.read();
        let editor_tab = app_state.editor_tab(self.panel_index, self.tab_index);
        editor_tab
            .editor
            .selected
            .map(|highlight| highlight.0 != highlight.1)
            .unwrap_or_default()
    }

    /// Create a highlights attribute.
    pub fn highlights_attr(&self, editor_id: usize) -> AttributeValue {
        let app_state = self.radio.read();
        let editor_tab = app_state.editor_tab(self.panel_index, self.tab_index);
        AttributeValue::any_value(CustomAttributeValues::TextHighlights(
            editor_tab
                .editor
                .get_visible_selection(editor_id)
                .map(|v| vec![v])
                .unwrap_or_default(),
        ))
    }

    /// Process a [`EditableEvent`] event.
    pub fn process_event(&mut self, edit_event: &EditableEvent) {
        let res = match edit_event {
            EditableEvent::MouseDown(e, id) => {
                let coords = e.get_element_coordinates();

                self.dragging.write().set_cursor_coords(coords);

                let mut app_state = self.radio.write();
                let editor_tab = app_state.editor_tab_mut(self.panel_index, self.tab_index);
                editor_tab.editor.clear_selection();

                Some((*id, Some(coords), None))
            }
            EditableEvent::MouseOver(e, id) => {
                if let Some(src) = self.dragging.peek().get_cursor_coords() {
                    let new_dist = e.get_element_coordinates();

                    Some((*id, None, Some((src, new_dist))))
                } else {
                    None
                }
            }
            EditableEvent::Click => {
                let selection = &mut *self.dragging.write();
                match selection {
                    TextDragging::FromCursorToPoint { shift, clicked, .. } if *shift => {
                        *clicked = false;
                    }
                    _ => {
                        *selection = TextDragging::None;
                    }
                }
                None
            }
            EditableEvent::KeyDown(e) => {
                if e.code == Code::ShiftLeft {
                    let dragging = &mut *self.dragging.write();
                    match dragging {
                        TextDragging::FromCursorToPoint {
                            shift: shift_pressed,
                            ..
                        } => {
                            *shift_pressed = true;
                        }
                        TextDragging::None => {
                            let app_state = self.radio.read();
                            let editor_tab = app_state.editor_tab(self.panel_index, self.tab_index);
                            *dragging = TextDragging::FromCursorToPoint {
                                shift: true,
                                clicked: false,
                                cursor: editor_tab.editor.cursor_pos(),
                                dist: None,
                            }
                        }
                        _ => {}
                    }
                }

                let is_plus = e.key == Key::Character("+".to_string());
                let is_minus = e.key == Key::Character("-".to_string());
                let is_e = e.code == Code::KeyE;
                let is_s = e.code == Code::KeyS;

                if e.code == Code::Escape
                    || (e.modifiers.contains(Modifiers::ALT) && (is_plus || is_minus || is_e))
                    || (e.modifiers.contains(Modifiers::CONTROL) && is_s)
                {
                    return;
                }

                let mut app_state = self.radio.write();
                let editor_tab = app_state.editor_tab_mut(self.panel_index, self.tab_index);
                let event = editor_tab.editor.process_key(&e.key, &e.code, &e.modifiers);
                if event.contains(TextEvent::TEXT_CHANGED) {
                    editor_tab.editor.run_parser();
                    *self.dragging.write() = TextDragging::None;
                } else if event.contains(TextEvent::SELECTION_CHANGED) {
                    self.dragging.write();
                }

                None
            }
            EditableEvent::KeyUp(e) => {
                if e.code == Code::ShiftLeft {
                    if let TextDragging::FromCursorToPoint { shift, .. } =
                        &mut *self.dragging.write()
                    {
                        *shift = false;
                    }
                } else {
                    *self.dragging.write() = TextDragging::None;
                }

                None
            }
        };

        if let Some((cursor_id, cursor_position, cursor_selection)) = res {
            if self.dragging.peek().has_cursor_coords() {
                self.platform
                    .send(EventMessage::RemeasureTextGroup(TextGroupMeasurement {
                        text_id: self.cursor_reference.peek().text_id,
                        cursor_id,
                        cursor_position,
                        cursor_selection,
                    }))
                    .unwrap()
            }
        }
    }
}

pub fn use_edit(radio: &RadioAppState, panel_index: usize, tab_index: usize) -> UseEdit {
    let dragging = use_signal(|| TextDragging::None);
    let platform = use_platform();
    let mut cursor_receiver_task = use_signal::<Option<Task>>(|| None);

    let cursor_reference = use_memo(use_reactive(&(panel_index, tab_index), {
        to_owned![radio];
        move |(panel_index, tab_index)| {
            if let Some(cursor_receiver_task) = cursor_receiver_task.write_unchecked().take() {
                cursor_receiver_task.cancel();
            }

            let text_id = Uuid::new_v4();
            let (cursor_sender, mut cursor_receiver) = unbounded_channel::<CursorLayoutResponse>();

            let cursor_reference = CursorReference {
                text_id,
                cursor_sender: cursor_sender.clone(),
            };

            let task = spawn(async move {
                while let Some(message) = cursor_receiver.recv().await {
                    match message {
                        // Update the cursor position calculated by the layout
                        CursorLayoutResponse::CursorPosition { position, id } => {
                            let mut app_state = radio.write();
                            let editor_tab = app_state.editor_tab(panel_index, tab_index);

                            let new_current_line = editor_tab.editor.rope.line(id);

                            // Use the line lenght as new column if the clicked column surpases the length
                            let new_cursor = if position >= new_current_line.chars().len() {
                                (
                                    editor_tab.editor.utf16_cu_to_char(
                                        new_current_line.as_str().unwrap().encode_utf16().count(),
                                    ),
                                    id,
                                )
                            } else {
                                (editor_tab.editor.utf16_cu_to_char(position), id)
                            };

                            // Only update if it's actually different
                            if editor_tab.editor.cursor.as_tuple() != new_cursor {
                                let editor_tab = app_state.editor_tab_mut(panel_index, tab_index);
                                editor_tab.editor.cursor.set_col(new_cursor.0);
                                editor_tab.editor.cursor.set_row(new_cursor.1);

                                if let TextDragging::FromCursorToPoint { cursor: from, .. } =
                                    dragging()
                                {
                                    let to = editor_tab.editor.cursor_pos();
                                    editor_tab.editor.set_selection((from, to));
                                } else {
                                    editor_tab.editor.clear_selection();
                                }
                            }
                        }
                        // Update the text selections calculated by the layout
                        CursorLayoutResponse::TextSelection { from, to, id } => {
                            let mut app_state = radio.write();
                            let editor_tab = app_state.editor_tab(panel_index, tab_index);

                            let current_cursor = editor_tab.editor.cursor().clone();
                            let current_selection = editor_tab.editor.get_selection();

                            let maybe_new_cursor = editor_tab.editor.measure_new_cursor(to, id);
                            let (from, to) = (
                                editor_tab.editor.utf16_cu_to_char(from),
                                editor_tab.editor.utf16_cu_to_char(to),
                            );
                            let maybe_new_selection =
                                editor_tab.editor.measure_new_selection(from, to, id);

                            // Update the text selection if it has changed
                            if let Some(current_selection) = current_selection {
                                if current_selection != maybe_new_selection {
                                    let editor_tab =
                                        app_state.editor_tab_mut(panel_index, tab_index);
                                    editor_tab.editor.set_selection(maybe_new_selection);
                                }
                            } else {
                                let editor_tab = app_state.editor_tab_mut(panel_index, tab_index);
                                editor_tab.editor.set_selection(maybe_new_selection);
                            }

                            // Update the cursor if it has changed
                            if current_cursor != maybe_new_cursor {
                                let editor_tab = app_state.editor_tab_mut(panel_index, tab_index);
                                *editor_tab.editor.cursor_mut() = maybe_new_cursor;
                            }
                        }
                    }
                }
            });

            cursor_receiver_task.set(Some(task));

            cursor_reference
        }
    }));

    UseEdit {
        radio: *radio,
        cursor_reference,
        dragging,
        platform,
        panel_index,
        tab_index,
    }
}
