use dioxus::{dioxus_core::AttributeValue, prelude::use_memo};
use dioxus_sdk::clipboard::UseClipboard;
use freya::prelude::{
    keyboard::{Code, Key, Modifiers},
    *,
};
use freya_common::{CursorLayoutResponse, EventMessage};
use freya_node_state::CursorReference;
use lsp_types::Url;
use ropey::iter::Lines;
use std::{
    cmp::Ordering,
    fmt::Display,
    ops::Range,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::unbounded_channel;
use uuid::Uuid;

use crate::{fs::FSTransport, lsp::LanguageId, state::RadioAppState};

use super::UseMetrics;

/// Iterator over text lines.
pub struct LinesIterator<'a> {
    lines: Lines<'a>,
}

impl<'a> Iterator for LinesIterator<'a> {
    type Item = Line<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.lines.next();

        line.map(|line| Line { text: line.into() })
    }
}

#[derive(Clone, PartialEq)]
pub enum EditorType {
    #[allow(dead_code)]
    Memory {
        title: String,
        id: String,
    },
    FS {
        path: PathBuf,
        root_path: PathBuf,
    },
}

impl EditorType {
    pub fn title_and_id(&self) -> (String, String) {
        match self {
            Self::Memory { title, id } => (title.clone(), id.clone()),
            Self::FS { path, .. } => (
                path.file_name().unwrap().to_str().unwrap().to_owned(),
                path.to_str().unwrap().to_owned(),
            ),
        }
    }

    pub fn paths(&self) -> Option<(&PathBuf, &PathBuf)> {
        match self {
            #[allow(unused_variables)]
            Self::Memory { title, id } => None,
            Self::FS { path, root_path } => Some((path, root_path)),
        }
    }

    pub fn language_id(&self) -> LanguageId {
        if let Some(ext) = self.paths().and_then(|(path, _)| path.extension()) {
            LanguageId::parse(ext.to_str().unwrap())
        } else {
            LanguageId::default()
        }
    }
}

#[derive(Clone)]
pub struct EditorData {
    pub(crate) editor_type: EditorType,
    pub(crate) cursor: TextCursor,
    pub(crate) history: EditorHistory,
    pub(crate) rope: Rope,
    pub(crate) selected: Option<(usize, usize)>,
    pub(crate) clipboard: UseClipboard,
    pub(crate) last_saved_history_change: usize,
    pub(crate) transport: FSTransport,
}

impl EditorData {
    pub fn new(
        editor_type: EditorType,
        rope: Rope,
        (row, col): (usize, usize),
        clipboard: UseClipboard,
        transport: FSTransport,
    ) -> Self {
        Self {
            editor_type,
            rope,
            cursor: TextCursor::new(row, col),
            selected: None,
            history: EditorHistory::new(),
            last_saved_history_change: 0,
            clipboard,
            transport,
        }
    }

    pub fn uri(&self) -> Option<Url> {
        self.editor_type
            .paths()
            .and_then(|(path, _)| Url::from_file_path(path).ok())
    }

    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    pub fn move_cursor_to_idx(&mut self, idx: usize) {
        let row = self.rope.byte_to_line(idx);
        let line_idx = self.rope.line_to_byte(row);
        let col = idx - line_idx;
        self.cursor_mut().move_to(row, col);
    }

    pub fn is_edited(&self) -> bool {
        self.history.current_change() != self.last_saved_history_change
    }

    pub fn mark_as_saved(&mut self) {
        self.last_saved_history_change = self.history.current_change();
    }

    pub fn path(&self) -> Option<&PathBuf> {
        self.editor_type.paths().map(|(path, _)| path)
    }

    pub fn cursor(&self) -> TextCursor {
        self.cursor.clone()
    }

    pub fn rope(&self) -> &Rope {
        &self.rope
    }
}

impl Display for EditorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.rope.to_string())
    }
}

impl TextEditor for EditorData {
    type LinesIterator<'a> = LinesIterator<'a>
    where
        Self: 'a;

    fn lines(&self) -> Self::LinesIterator<'_> {
        let lines = self.rope.lines();
        LinesIterator { lines }
    }

    fn insert_char(&mut self, char: char, char_idx: usize) {
        self.history.push_change(HistoryChange::InsertChar {
            idx: char_idx,
            char,
        });
        self.rope.insert_char(char_idx, char);
    }

    fn insert(&mut self, text: &str, idx: usize) {
        self.history.push_change(HistoryChange::InsertText {
            idx,
            text: text.to_owned(),
        });
        self.rope.insert(idx, text);
    }

    fn remove(&mut self, range: Range<usize>) {
        let text = self.rope.slice(range.clone()).to_string();
        self.history.push_change(HistoryChange::Remove {
            idx: range.start,
            text,
        });
        self.rope.remove(range)
    }

    fn char_to_line(&self, char_idx: usize) -> usize {
        self.rope.char_to_line(char_idx)
    }

    fn line_to_char(&self, line_idx: usize) -> usize {
        self.rope.line_to_char(line_idx)
    }

    fn line(&self, line_idx: usize) -> Option<Line<'_>> {
        let line = self.rope.get_line(line_idx);

        line.map(|line| Line { text: line.into() })
    }

    fn len_lines<'a>(&self) -> usize {
        self.rope.len_lines()
    }

    fn cursor(&self) -> &TextCursor {
        &self.cursor
    }

    fn cursor_mut(&mut self) -> &mut TextCursor {
        &mut self.cursor
    }

    fn move_highlight_to_cursor(&mut self) {
        let pos = self.cursor_pos();
        if let Some(selected) = self.selected.as_mut() {
            selected.1 = pos;
        } else {
            self.selected = Some((self.cursor_pos(), self.cursor_pos()))
        }
    }

    fn has_any_highlight(&self) -> bool {
        self.selected.is_some()
    }

    fn highlights(&self, editor_id: usize) -> Option<(usize, usize)> {
        let (selected_from, selected_to) = self.selected?;

        let selected_to_row = self.char_to_line(selected_to);
        let selected_from_row = self.char_to_line(selected_from);

        let selected_to_line = self.char_to_line(selected_to);
        let selected_from_line = self.char_to_line(selected_from);

        let editor_row_idx = self.line_to_char(editor_id);
        let selected_to_row_idx = self.line_to_char(selected_to_line);
        let selected_from_row_idx = self.line_to_char(selected_from_line);

        let selected_to_col_idx = selected_to - selected_to_row_idx;
        let selected_from_col_idx = selected_from - selected_from_row_idx;

        // Between starting line and endling line
        if (editor_id > selected_from_row && editor_id < selected_to_row)
            || (editor_id < selected_from_row && editor_id > selected_to_row)
        {
            let len = self.line(editor_id).unwrap().len_chars();
            return Some((0, len));
        }

        match selected_from_row.cmp(&selected_to_row) {
            // Selection direction is from bottom -> top
            Ordering::Greater => {
                if selected_from_row == editor_id {
                    // Starting line
                    return Some((0, selected_from_col_idx));
                } else if selected_to_row == editor_id {
                    // Ending line
                    let len = self.line(selected_to_row).unwrap().len_chars();
                    return Some((selected_to_col_idx, len));
                }
            }
            // Selection direction is from top -> bottom
            Ordering::Less => {
                if selected_from_row == editor_id {
                    // Starting line
                    let len = self.line(selected_from_row).unwrap().len_chars();
                    return Some((selected_from_col_idx, len));
                } else if selected_to_row == editor_id {
                    // Ending line
                    return Some((0, selected_to_col_idx));
                }
            }
            Ordering::Equal => {
                // Starting and endline line are the same
                if selected_from_row == editor_id {
                    return Some((selected_from - editor_row_idx, selected_to - editor_row_idx));
                }
            }
        }

        None
    }

    fn set(&mut self, text: &str) {
        self.rope.remove(0..);
        self.rope.insert(0, text);
    }

    fn unhighlight(&mut self) {
        self.selected = None;
    }

    fn highlight_text(&mut self, from: usize, to: usize, editor_id: usize) {
        let row_idx = self.line_to_char(editor_id);
        if self.selected.is_none() {
            self.selected = Some((row_idx + from, row_idx + to));
        } else {
            self.selected.as_mut().unwrap().1 = row_idx + to;
        }

        self.cursor_mut().move_to(editor_id, to);
    }

    fn get_clipboard(&mut self) -> &mut UseClipboard {
        &mut self.clipboard
    }

    fn get_selected_text(&self) -> Option<String> {
        let (start, end) = self.get_selection()?;

        Some(self.rope().get_slice(start..end)?.to_string())
    }

    fn get_selection(&self) -> Option<(usize, usize)> {
        let (start, end) = self.selected?;

        // Use left-to-right selection
        let (start, end) = if start < end {
            (start, end)
        } else {
            (end, start)
        };

        Some((start, end))
    }

    fn redo(&mut self) -> Option<usize> {
        if self.history.can_redo() {
            let cursor_idx = self.history.redo(&mut self.rope);
            if let Some(cursor_idx) = cursor_idx {
                self.move_cursor_to_idx(cursor_idx);
            }

            cursor_idx
        } else {
            None
        }
    }

    fn undo(&mut self) -> Option<usize> {
        if self.history.can_undo() {
            let cursor_idx = self.history.undo(&mut self.rope);
            if let Some(cursor_idx) = cursor_idx {
                self.move_cursor_to_idx(cursor_idx);
            }
            cursor_idx
        } else {
            None
        }
    }
}

/// Manage an editable content.
#[derive(Clone, Copy, PartialEq)]
pub struct UseEdit {
    pub(crate) radio: RadioAppState,
    pub(crate) cursor_reference: Memo<CursorReference>,
    pub(crate) selecting_text_with_mouse: Signal<Option<CursorPoint>>,
    pub(crate) platform: UsePlatform,
    pub(crate) pane_index: usize,
    pub(crate) editor_index: usize,
    pub(crate) metrics: UseMetrics,
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
        let editor = app_state
            .panel(self.pane_index)
            .tab(self.editor_index)
            .as_text_editor()
            .unwrap();
        AttributeValue::any_value(CustomAttributeValues::TextHighlights(
            editor
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

                let editor = app_state.editor_mut(self.pane_index, self.editor_index);
                editor.unhighlight();
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

                let event = 'key_matcher: {
                    let mut app_state = self.radio.write();
                    let editor = app_state.editor_mut(self.pane_index, self.editor_index);

                    if e.modifiers.contains(Modifiers::CONTROL) {
                        if e.code == Code::KeyZ {
                            editor.undo();
                            break 'key_matcher TextEvent::TEXT_CHANGED;
                        } else if e.code == Code::KeyY {
                            editor.redo();
                            break 'key_matcher TextEvent::TEXT_CHANGED;
                        }
                    }

                    editor.process_key(&e.key, &e.code, &e.modifiers)
                };
                if event.contains(TextEvent::TEXT_CHANGED) {
                    self.metrics.run_metrics();
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

pub fn use_edit(
    radio: &RadioAppState,
    pane_index: usize,
    editor_index: usize,
    metrics: &UseMetrics,
) -> UseEdit {
    let selecting_text_with_mouse = use_signal(|| None);
    let platform = use_platform();
    let mut cursor_receiver_task = use_signal::<Option<Task>>(|| None);

    let cursor_reference = use_memo(use_reactive(&(pane_index, editor_index), {
        to_owned![radio];
        move |(pane_index, editor_index)| {
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
                                let editor = app_state.editor(pane_index, editor_index);

                                let new_current_line = editor.rope.line(id);

                                // Use the line lenght as new column if the clicked column surpases the length
                                let new_cursor = if position >= new_current_line.chars().len() {
                                    (new_current_line.chars().len(), id)
                                } else {
                                    (position, id)
                                };

                                // Only update if it's actually different
                                if editor.cursor.as_tuple() != new_cursor {
                                    let editor = app_state.editor_mut(pane_index, editor_index);
                                    editor.cursor.set_col(new_cursor.0);
                                    editor.cursor.set_row(new_cursor.1);
                                    editor.unhighlight();
                                }

                                // Remove the current calcutions so the layout engine doesn't try to calculate again
                                cursor_reference.set_cursor_position(None);
                            }
                            // Update the text selections calculated by the layout
                            CursorLayoutResponse::TextSelection { from, to, id } => {
                                let mut app_state = radio.write();
                                let editor = app_state.editor_mut(pane_index, editor_index);
                                editor.highlight_text(from, to, id);
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
        pane_index,
        editor_index,
        metrics: *metrics,
    }
}
