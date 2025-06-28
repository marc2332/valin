use dioxus::dioxus_core::AttributeValue;
use dioxus_radio::prelude::ChannelSelection;

use crate::{
    state::TabId,
    views::panels::tabs::editor::{AppStateEditorUtils, EditorTab},
};
use freya::{
    core::custom_attributes::{CursorLayoutResponse, CursorReference},
    prelude::*,
};
use tokio::sync::mpsc::unbounded_channel;

use crate::state::RadioAppState;

/// Manage an editable content.
#[derive(Clone, Copy, PartialEq)]
pub struct UseEdit {
    pub(crate) cursor_reference: CopyValue<CursorReference>,
}

impl UseEdit {
    /// Create a cursor attribute.
    pub fn cursor_attr(&self) -> AttributeValue {
        AttributeValue::any_value(CustomAttributeValues::CursorReference(
            self.cursor_reference.read().clone(),
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
}

pub fn use_edit(mut radio: RadioAppState, tab_id: TabId, text_id: usize) -> UseEdit {
    use_hook(|| {
        let (cursor_sender, mut cursor_receiver) = unbounded_channel::<CursorLayoutResponse>();

        let cursor_reference = CopyValue::new(CursorReference {
            text_id,
            cursor_sender,
        });

        spawn(async move {
            while let Some(message) = cursor_receiver.recv().await {
                match message {
                    // Update the cursor position calculated by the layout
                    CursorLayoutResponse::CursorPosition { position, id } => {
                        radio.write_with_channel_selection(|app_state| {
                            let editor_tab = app_state.editor_tab(tab_id);

                            let new_cursor = editor_tab.editor.measure_new_cursor(
                                editor_tab.editor.utf16_cu_to_char(position),
                                id,
                            );

                            // Only update and clear the selection if the cursor has changed
                            if editor_tab.editor.cursor() != new_cursor {
                                let editor_tab = app_state.editor_tab_mut(tab_id);
                                *editor_tab.editor.cursor_mut() = new_cursor;
                                if let TextDragging::FromCursorToPoint { cursor: from, .. } =
                                    &editor_tab.editor.dragging
                                {
                                    let to = editor_tab.editor.cursor_pos();
                                    editor_tab.editor.set_selection((*from, to));
                                } else {
                                    editor_tab.editor.clear_selection();
                                }
                                ChannelSelection::Current
                            } else {
                                ChannelSelection::Silence
                            }
                        });
                    }
                    // Update the text selections calculated by the layout
                    CursorLayoutResponse::TextSelection { from, to, id } => {
                        radio.write_with_channel_selection(|app_state| {
                            let mut channel = ChannelSelection::Silence;

                            let editor_tab = app_state.editor_tab_mut(tab_id);

                            let current_cursor = editor_tab.editor.cursor();
                            let current_selection = editor_tab.editor.get_selection();

                            let maybe_new_cursor = editor_tab.editor.measure_new_cursor(to, id);
                            let maybe_new_selection =
                                editor_tab.editor.measure_new_selection(from, to, id);

                            // Update the text selection if it has changed
                            if let Some(current_selection) = current_selection {
                                if current_selection != maybe_new_selection {
                                    editor_tab.editor.set_selection(maybe_new_selection);
                                    channel.current();
                                }
                            } else {
                                editor_tab.editor.set_selection(maybe_new_selection);
                                channel.current();
                            }

                            // Update the cursor if it has changed
                            if current_cursor != maybe_new_cursor {
                                *editor_tab.editor.cursor_mut() = maybe_new_cursor;
                                channel.current();
                            }

                            channel
                        });
                    }
                }
            }
        });

        UseEdit { cursor_reference }
    })
}
