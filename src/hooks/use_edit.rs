use dioxus::{dioxus_core::AttributeValue, prelude::use_memo};

use crate::tabs::editor::AppStateEditorUtils;
use freya::common::{CursorLayoutResponse, EventMessage};
use freya::prelude::{
    keyboard::{Key, Modifiers},
    *,
};
use freya_node_state::CursorReference;

use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::unbounded_channel;
use uuid::Uuid;

use crate::state::RadioAppState;

/// Manage an editable content.
#[derive(Clone, Copy, PartialEq)]
pub struct UseEdit {
    pub(crate) radio: RadioAppState,
    pub(crate) cursor_reference: Memo<CursorReference>,
    pub(crate) selecting_text_with_mouse: Signal<Option<CursorPoint>>,
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

    /// Create a highlights attribute.
    pub fn highlights_attr(&self, editor_id: usize) -> AttributeValue {
        let app_state = self.radio.read();
        let editor_tab = app_state.editor_tab(self.panel_index, self.tab_index);
        AttributeValue::any_value(CustomAttributeValues::TextHighlights(
            editor_tab
                .editor
                .highlights(editor_id)
                .map(|v| vec![v])
                .unwrap_or_default(),
        ))
    }

    /// Process a [`EditableEvent`] event.
    pub fn process_event(&mut self, edit_event: &EditableEvent) {
        match edit_event {
            EditableEvent::MouseDown(e, id) => {
                let coords = e.get_element_coordinates();
                *self.selecting_text_with_mouse.write() = Some(coords);

                self.cursor_reference.read().set_id(Some(*id));
                self.cursor_reference
                    .read()
                    .set_cursor_position(Some(coords));
                let mut app_state = self.radio.write();

                let editor_tab = app_state.editor_tab_mut(self.panel_index, self.tab_index);
                editor_tab.editor.unhighlight();
            }
            EditableEvent::MouseOver(e, id) => {
                self.selecting_text_with_mouse.with(|selecting_text| {
                    if let Some(current_dragging) = selecting_text {
                        let coords = e.get_element_coordinates();

                        self.cursor_reference.read().set_id(Some(*id));
                        self.cursor_reference
                            .read()
                            .set_cursor_selections(Some((*current_dragging, coords)));
                    }
                });
            }
            EditableEvent::Click => {
                *self.selecting_text_with_mouse.write() = None;
            }
            EditableEvent::KeyDown(e) => {
                let is_plus = e.key == Key::Character("+".to_string());
                let is_minus = e.key == Key::Character("-".to_string());
                let is_e = e.key == Key::Character("e".to_string());
                let is_s = e.key == Key::Character("s".to_string());

                if e.key == Key::Escape
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
                    *self.selecting_text_with_mouse.write() = None;
                } else if event.contains(TextEvent::SELECTION_CHANGED) {
                    self.selecting_text_with_mouse.write();
                }
            }
        }

        if self.selecting_text_with_mouse.peek().is_some() {
            self.platform
                .send(EventMessage::RemeasureTextGroup(
                    self.cursor_reference.read().text_id,
                ))
                .unwrap();
        }
    }
}

pub fn use_edit(radio: &RadioAppState, panel_index: usize, tab_index: usize) -> UseEdit {
    let selecting_text_with_mouse = use_signal(|| None);
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
                cursor_position: Arc::new(Mutex::new(None)),
                cursor_id: Arc::new(Mutex::new(None)),
                cursor_selections: Arc::new(Mutex::new(None)),
            };

            let task = spawn({
                to_owned![cursor_reference];
                async move {
                    while let Some(message) = cursor_receiver.recv().await {
                        match message {
                            // Update the cursor position calculated by the layout
                            CursorLayoutResponse::CursorPosition { position, id } => {
                                let mut app_state = radio.write();
                                let editor_tab = app_state.editor_tab(panel_index, tab_index);

                                let new_current_line = editor_tab.editor.rope.line(id);

                                // Use the line lenght as new column if the clicked column surpases the length
                                let new_cursor = if position >= new_current_line.chars().len() {
                                    (new_current_line.chars().len(), id)
                                } else {
                                    (position, id)
                                };

                                // Only update if it's actually different
                                if editor_tab.editor.cursor.as_tuple() != new_cursor {
                                    let editor_tab =
                                        app_state.editor_tab_mut(panel_index, tab_index);
                                    editor_tab.editor.cursor.set_col(new_cursor.0);
                                    editor_tab.editor.cursor.set_row(new_cursor.1);
                                    editor_tab.editor.unhighlight();
                                }

                                // Remove the current calcutions so the layout engine doesn't try to calculate again
                                cursor_reference.set_cursor_position(None);
                            }
                            // Update the text selections calculated by the layout
                            CursorLayoutResponse::TextSelection { from, to, id } => {
                                let mut app_state = radio.write();
                                let editor_tab = app_state.editor_tab_mut(panel_index, tab_index);
                                editor_tab.editor.highlight_text(from, to, id);
                                cursor_reference.set_cursor_selections(None);
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
        selecting_text_with_mouse,
        platform,
        panel_index,
        tab_index,
    }
}
