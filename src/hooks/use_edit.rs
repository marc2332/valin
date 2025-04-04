use dioxus::dioxus_core::AttributeValue;

use crate::views::panels::tabs::editor::{AppStateEditorUtils, EditorTab};
use freya::{
    core::{
        custom_attributes::{CursorLayoutResponse, CursorReference},
        event_loop_messages::{EventLoopMessage, TextGroupMeasurement},
    },
    prelude::*,
};
use tokio::sync::mpsc::unbounded_channel;
use uuid::Uuid;

use crate::state::{Channel, RadioAppState};

/// Manage an editable content.
#[derive(Clone, Copy, PartialEq)]
pub struct UseEdit {
    pub(crate) cursor_reference: Memo<CursorReference>,
    pub(crate) dragging: Signal<TextDragging>,
    pub(crate) platform: UsePlatform,
}

impl UseEdit {
    /// Create a cursor attribute.
    pub fn cursor_attr(&self) -> AttributeValue {
        AttributeValue::any_value(CustomAttributeValues::CursorReference(
            self.cursor_reference.peek().clone(),
        ))
    }

    /// Check if there is any highlight at all.
    pub fn has_any_highlight(&self, editor_tab: &EditorTab) -> bool {
        editor_tab
            .editor
            .selected
            .map(|highlight| highlight.0 != highlight.1)
            .unwrap_or_default()
    }

    /// Create a highlights attribute.
    pub fn highlights_attr(&self, editor_id: usize, editor_tab: &EditorTab) -> AttributeValue {
        AttributeValue::any_value(CustomAttributeValues::TextHighlights(
            editor_tab
                .editor
                .get_visible_selection(editor_id)
                .map(|v| vec![v])
                .unwrap_or_default(),
        ))
    }

    /// Process a [`EditableEvent`] event.
    pub fn process_event(
        &mut self,
        edit_event: &EditableEvent,
        editor_tab: &mut EditorTab,
    ) -> bool {
        let res = match edit_event {
            EditableEvent::MouseDown(e, id) => {
                let coords = e.get_element_coordinates();

                self.dragging.write().set_cursor_coords(coords);
                editor_tab.editor.clear_selection();

                Some((*id, Some(coords), None))
            }
            EditableEvent::MouseMove(e, id) => {
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
                    .send(EventLoopMessage::RemeasureTextGroup(TextGroupMeasurement {
                        text_id: self.cursor_reference.peek().text_id,
                        cursor_id,
                        cursor_position,
                        cursor_selection,
                    }))
                    .unwrap()
            }
            true
        } else {
            false
        }
    }
}

pub fn use_edit(mut radio: RadioAppState, panel_index: usize, tab_index: usize) -> UseEdit {
    let dragging = use_signal(|| TextDragging::None);
    let platform = use_platform();
    let mut cursor_receiver_task = use_signal::<Option<Task>>(|| None);
    let tab_channel = Channel::follow_tab(panel_index, tab_index);

    let cursor_reference = use_memo(use_reactive(&(panel_index, tab_index), {
        move |(panel_index, tab_index)| {
            if let Some(cursor_receiver_task) = cursor_receiver_task.write_unchecked().take() {
                cursor_receiver_task.cancel();
            }

            let text_id = Uuid::new_v4();
            let (cursor_sender, mut cursor_receiver) = unbounded_channel::<CursorLayoutResponse>();

            let cursor_reference = CursorReference {
                text_id,
                cursor_sender,
            };

            let task = spawn(async move {
                while let Some(message) = cursor_receiver.recv().await {
                    match message {
                        // Update the cursor position calculated by the layout
                        CursorLayoutResponse::CursorPosition { position, id } => {
                            radio.write_with_map_channel(|app_state| {
                                let editor_tab = app_state.editor_tab(panel_index, tab_index);

                                let new_cursor = editor_tab.editor.measure_new_cursor(
                                    editor_tab.editor.utf16_cu_to_char(position),
                                    id,
                                );

                                // Only update and clear the selection if the cursor has changed
                                if editor_tab.editor.cursor() != new_cursor {
                                    let editor_tab =
                                        app_state.editor_tab_mut(panel_index, tab_index);
                                    *editor_tab.editor.cursor_mut() = new_cursor;
                                    if let TextDragging::FromCursorToPoint {
                                        cursor: from, ..
                                    } = &*dragging.read()
                                    {
                                        let to = editor_tab.editor.cursor_pos();
                                        editor_tab.editor.set_selection((*from, to));
                                    } else {
                                        editor_tab.editor.clear_selection();
                                    }
                                    tab_channel
                                } else {
                                    Channel::Void
                                }
                            });
                        }
                        // Update the text selections calculated by the layout
                        CursorLayoutResponse::TextSelection { from, to, id } => {
                            radio.write_with_map_channel(|app_state| {
                                let mut channel = Channel::Void;

                                let editor_tab = app_state.editor_tab_mut(panel_index, tab_index);

                                let current_cursor = editor_tab.editor.cursor().clone();
                                let current_selection = editor_tab.editor.get_selection();

                                let maybe_new_cursor = editor_tab.editor.measure_new_cursor(to, id);
                                let maybe_new_selection =
                                    editor_tab.editor.measure_new_selection(from, to, id);

                                // Update the text selection if it has changed
                                if let Some(current_selection) = current_selection {
                                    if current_selection != maybe_new_selection {
                                        editor_tab.editor.set_selection(maybe_new_selection);
                                        channel = tab_channel;
                                    }
                                } else {
                                    editor_tab.editor.set_selection(maybe_new_selection);
                                    channel = tab_channel;
                                }

                                // Update the cursor if it has changed
                                if current_cursor != maybe_new_cursor {
                                    *editor_tab.editor.cursor_mut() = maybe_new_cursor;
                                    channel = tab_channel;
                                }

                                channel
                            });
                        }
                    }
                }
            });

            cursor_receiver_task.set(Some(task));

            cursor_reference
        }
    }));

    UseEdit {
        cursor_reference,
        dragging,
        platform,
    }
}
