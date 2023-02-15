use freya::prelude::*;
use freya_node_state::CursorReference;
use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};
use tokio::sync::{mpsc::unbounded_channel, mpsc::UnboundedSender};

use crate::{EditorData, EditorManager};

/// How the editable content must behave.
pub enum EditableMode {
    /// Multiple editors of only one line.
    ///
    /// Useful for textarea-like editors that need more customization than a simple paragraph for example.
    SingleLineMultipleEditors,
    /// One editor of multiple lines.
    ///
    /// A paragraph for example.
    MultipleLinesSingleEditor,
}

pub type KeypressNotifier = UnboundedSender<Rc<KeyboardData>>;
pub type ClickNotifier = UnboundedSender<(Rc<MouseData>, usize)>;
pub type EditableText = UseState<Vec<Arc<Mutex<EditorData>>>>;

pub fn use_edit<'a>(
    cx: &'a ScopeState,
    mode: EditableMode,
    manager: &UseState<EditorManager>,
    pane_index: usize,
    editor: usize,
    highlight_trigger: UnboundedSender<()>,
) -> (KeypressNotifier, ClickNotifier, AttributeValue<'a>) {
    let cursor_channels = use_ref(cx, || {
        let (tx, rx) = unbounded_channel::<(usize, usize)>();
        (tx, Some(rx))
    });

    // editor.cursorreference passed to the layout engine
    let cursor_ref = use_ref(cx, || CursorReference {
        agent: cursor_channels.read().0.clone(),
        positions: Arc::new(Mutex::new(None)),
        id: Arc::new(Mutex::new(None)),
    });

    // This will allow to pass the editor.cursorreference as an attribute value
    let cursor_ref_attr = cx.any_value(CustomAttributeValues::CursorReference(
        cursor_ref.read().clone(),
    ));

    // Single listener multiple triggers channel so the mouse can be changed from multiple elements
    let click_channel = use_ref(cx, || {
        let (tx, rx) = unbounded_channel::<(Rc<MouseData>, usize)>();
        (tx, Some(rx))
    });

    // Single listener multiple triggers channel to write from different sources
    let keypress_channel = use_ref(cx, || {
        let (tx, rx) = unbounded_channel::<Rc<KeyboardData>>();
        (tx, Some(rx))
    });

    // Update the new positions and ID from the editor.cursorreference so the layout engine can make the proper calculations
    {
        let click_channel = click_channel.clone();
        let cursor_ref = cursor_ref.clone();
        use_effect(cx, &pane_index, move |_| {
            let click_channel = click_channel.clone();
            async move {
                let rx = click_channel.write().1.take();
                let mut rx = rx.unwrap();

                while let Some((e, id)) = rx.recv().await {
                    let points = e.get_element_coordinates();
                    let cursor_ref = cursor_ref.clone();
                    cursor_ref.write().id.lock().unwrap().replace(id);
                    cursor_ref
                        .write()
                        .positions
                        .lock()
                        .unwrap()
                        .replace((points.x as f32, points.y as f32));
                }
            }
        });
    }

    // Listen for new calculations from the layout engine
    use_effect(cx, &pane_index, move |_| {
        let cursor_ref = cursor_ref.clone();
        let manager = manager.clone();
        let cursor_channels = cursor_channels.clone();
        async move {
            let cursor_receiver = cursor_channels.write().1.take();
            let mut cursor_receiver = cursor_receiver.unwrap();
            let cursor_ref = cursor_ref.clone();

            while let Some((new_index, editor_num)) = cursor_receiver.recv().await {
                manager.with_mut(|manager| {
                    let mut editor = manager.panes[pane_index].editors[editor].lock().unwrap();

                    let new_cursor_row = match mode {
                        EditableMode::MultipleLinesSingleEditor => {
                            editor.rope.line(new_index).chars().len()
                        }
                        EditableMode::SingleLineMultipleEditors => editor_num,
                    };

                    let new_cursor_col = match mode {
                        EditableMode::MultipleLinesSingleEditor => {
                            new_index - editor.rope.line_to_char(new_cursor_row)
                        }
                        EditableMode::SingleLineMultipleEditors => new_index,
                    };

                    let new_current_line = editor.rope.line(new_cursor_row);

                    // Use the line lenght as new column if the clicked column surpases the length
                    let new_cursor = if new_cursor_col >= new_current_line.chars().len() {
                        (new_current_line.chars().len(), new_cursor_row)
                    } else {
                        (new_cursor_col, new_cursor_row)
                    };

                    // Only update if it's actually different
                    if editor.cursor != new_cursor {
                        editor.cursor = new_cursor;
                    }

                    // Remove the current calcutions so the layout engine doesn't try to calculate again
                    cursor_ref.write().positions.lock().unwrap().take();
                });
            }
        }
    });

    use_effect(cx, &pane_index, move |_| {
        let keypress_channel = keypress_channel.clone();
        let manager = manager.clone();
        async move {
            let rx = keypress_channel.write().1.take();
            let mut rx = rx.unwrap();

            while let Some(e) = rx.recv().await {
                manager.with_mut(|manager| {
                    let mut editor = manager.panes[pane_index].editors[editor].lock().unwrap();

                    match &e.key {
                        Key::ArrowDown => {
                            let total_lines = editor.rope.len_lines();
                            // Go one line down
                            if editor.cursor.1 < total_lines - 1 {
                                let next_line =
                                    editor.rope.line(editor.cursor.1 + 1);

                                // Try to use the current editor.cursorcolumn, otherwise use the new line length
                                let cursor_index = if editor.cursor.0 <= next_line.chars().len() {
                                    editor.cursor.0
                                } else {
                                    next_line.chars().len()
                                };

                                editor.cursor.0 = cursor_index;
                                editor.cursor.1 += 1;
                            }
                        }
                        Key::ArrowLeft => {
                            // Go one character to the left
                            if editor.cursor.0 > 0 {
                                editor.cursor.0 -= 1;
                            } else if editor.cursor.1 > 0 {
                                // Go one line up if there is no more characters on the left
                                let prev_line = editor.rope.get_line(editor.cursor.1 - 1);
                                if let Some(prev_line) = prev_line {
                                    // Use the new line length as new editor.cursorcolumn, otherwise just set it to 0
                                    let len = if prev_line.chars().len() > 0 {
                                        prev_line.chars().len()
                                    } else {
                                        0
                                    };
                                    editor.cursor = (len, editor.cursor.1 - 1);
                                }
                            }
                        }
                        Key::ArrowRight => {
                            let total_lines = editor.rope.len_lines();
                            let current_line = editor.rope.line(editor.cursor.1);

                            // Go one line down if there isn't more characters on the right
                            if editor.cursor.1 < total_lines - 1
                                && editor.cursor.0 == current_line.chars().len()
                            {
                                editor.cursor = (0, editor.cursor.1 + 1);
                            } else if editor.cursor.0 < current_line.chars().len() {
                                // Go one character to the right if possible
                                editor.cursor.0 += 1;
                            }
                        }
                        Key::ArrowUp => {
                            // Go one line up if there is any
                            if editor.cursor.1 > 0 {
                                let prev_line =
                                    editor.rope.line(editor.cursor.1 - 1);

                                // Try to use the current editor.cursorcolumn, otherwise use the new line length
                                let cursor_column = if editor.cursor.0 <= prev_line.chars().len() {
                                    editor.cursor.0
                                } else {
                                    prev_line.chars().len()
                                };

                                editor.cursor = (cursor_column, editor.cursor.1 - 1);
                            }
                        }
                        Key::Backspace => {
                            if editor.cursor.0 > 0 {
                                // Remove the character to the left if there is any
                                let char_idx =
                                    editor.rope.line_to_char(editor.cursor.1) + editor.cursor.0;
                                editor.rope.remove(char_idx - 1..char_idx);
                                editor.cursor.0 -= 1;

                                highlight_trigger.send(()).ok();
                            } else if editor.cursor.1 > 0 {
                                // Moves the whole current line to the end of the line above.
                                let prev_line_len = editor
                                    .rope
                                    .line(editor.cursor.1 - 1)
                                    .len_chars();
                                let current_line =
                                    editor.rope.get_line(editor.cursor.1).clone();

                                if let Some(current_line) = current_line {
                                    let prev_char_idx =
                                        editor.rope.line_to_char(editor.cursor.1 - 1) + prev_line_len - 1;
                                    let char_idx = editor.rope.line_to_char(editor.cursor.1)  + current_line.len_chars();

                                    let current_line = current_line.to_string();

                                    editor.rope.insert(prev_char_idx, &current_line);
                                    editor.rope.remove(char_idx..(char_idx + current_line.len()) + 1);
                                   
                                }
                                editor.cursor = (prev_line_len, editor.cursor.1 - 1);
                                
                                highlight_trigger.send(()).ok();
                            }
                        }
                        Key::Enter => {
                            // Breaks the line
                            let char_idx =
                                editor.rope.line_to_char(editor.cursor.1) + editor.cursor.0;

                            editor.rope.insert(char_idx, "\n");

                            editor.cursor = (0, editor.cursor.1 + 1);
                            highlight_trigger.send(()).ok();
                        }
                        Key::Character(character) => {
                            match e.code {
                                Code::Delete => {}
                                Code::Space => {
                                    // Simply adds an space
                                    let char_idx = editor.rope.line_to_char(editor.cursor.1)
                                        + editor.cursor.0;
                                    editor.rope.insert(char_idx, " ");
                                    editor.cursor.0 += 1;
                                }
                                _ => {
                                    // Adds a new character to the right
                                    let char_idx = editor.rope.line_to_char(editor.cursor.1)
                                        + editor.cursor.0;
                                    editor.rope.insert(char_idx, character);
                                    editor.cursor.0 += 1;
                                }
                            }
                            highlight_trigger.send(()).ok();
                        }
                        _ => {}
                    }
                    
                });
            }
        }
    });

    (
        keypress_channel.read().0.clone(),
        click_channel.read().0.clone(),
        cursor_ref_attr,
    )
}
