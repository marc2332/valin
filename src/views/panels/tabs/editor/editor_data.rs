use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::{cmp::Ordering, fmt::Display, ops::Range, path::PathBuf};

use dioxus_clipboard::prelude::UseClipboard;
use freya::core::event_loop_messages::{EventLoopMessage, TextGroupMeasurement};
use freya::events::Code;
use freya::hooks::{EditorHistory, HistoryChange, Line, LinesIterator, TextCursor, TextEditor};
use freya::prelude::Rope;
use freya_hooks::{EditableEvent, TextDragging, TextEvent, UsePlatform};
use lsp_types::Url;
use skia_safe::textlayout::FontCollection;
use uuid::Uuid;

use crate::{fs::FSTransport, lsp::LanguageId, metrics::EditorMetrics};

pub type SharedRope = Rc<RefCell<Rope>>;

#[derive(Clone, PartialEq)]
pub enum EditorType {
    #[allow(dead_code)]
    Memory {
        title: String,
    },
    FS {
        path: PathBuf,
        root_path: PathBuf,
    },
}

impl EditorType {
    pub fn title(&self) -> String {
        match self {
            Self::Memory { title } => title.clone(),
            Self::FS { path, .. } => path.file_name().unwrap().to_str().unwrap().to_owned(),
        }
    }

    pub fn paths(&self) -> Option<(&PathBuf, &PathBuf)> {
        match self {
            #[allow(unused_variables)]
            Self::Memory { title } => None,
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

pub struct EditorData {
    pub(crate) editor_type: EditorType,
    pub(crate) cursor: TextCursor,
    pub(crate) history: EditorHistory,
    pub(crate) rope: SharedRope,
    pub(crate) selected: Option<(usize, usize)>,
    pub(crate) clipboard: UseClipboard,
    pub(crate) last_saved_history_change: usize,
    pub(crate) transport: FSTransport,
    pub(crate) metrics: EditorMetrics,
    pub(crate) dragging: TextDragging,
    pub(crate) text_id: Uuid,
}

impl EditorData {
    pub fn new(
        editor_type: EditorType,
        rope: Rope,
        pos: usize,
        clipboard: UseClipboard,
        transport: FSTransport,
        font_size: f32,
        font_collection: &FontCollection,
    ) -> Self {
        let mut metrics = EditorMetrics::new();
        metrics.measure_longest_line(font_size, &rope, font_collection);
        metrics.run_parser(&rope);

        Self {
            editor_type,
            rope: Rc::new(RefCell::new(rope)),
            cursor: TextCursor::new(pos),
            selected: None,
            history: EditorHistory::new(),
            last_saved_history_change: 0,
            clipboard,
            transport,
            metrics,
            dragging: TextDragging::None,
            text_id: Uuid::new_v4(),
        }
    }

    pub fn uri(&self) -> Option<Url> {
        self.editor_type
            .paths()
            .and_then(|(path, _)| Url::from_file_path(path).ok())
    }

    pub fn text(&self) -> String {
        self.rope.borrow().to_string()
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

    pub fn rope(&self) -> &SharedRope {
        &self.rope
    }

    pub fn run_parser(&mut self) {
        self.metrics.run_parser(&self.rope.borrow());
    }

    pub fn measure_longest_line(&mut self, font_size: f32, font_collection: &FontCollection) {
        self.metrics
            .measure_longest_line(font_size, &self.rope.borrow(), font_collection);
    }

    pub fn editor_type(&self) -> &EditorType {
        &self.editor_type
    }

    pub fn process_event(&mut self, edit_event: &EditableEvent) -> bool {
        let mut processed = false;
        let res = match edit_event {
            EditableEvent::MouseDown(e, id) => {
                let coords = e.get_element_coordinates();

                self.dragging.set_cursor_coords(coords);
                self.clear_selection();
                processed = true;

                Some((*id, Some(coords), None))
            }
            EditableEvent::MouseMove(e, id) => {
                if let Some(src) = self.dragging.get_cursor_coords() {
                    let new_dist = e.get_element_coordinates();

                    Some((*id, None, Some((src, new_dist))))
                } else {
                    None
                }
            }
            EditableEvent::Click => {
                let selection = &mut self.dragging;
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
                    let cursor = self.cursor_pos();
                    let dragging = &mut self.dragging;
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
                                cursor,
                                dist: None,
                            }
                        }
                        _ => {}
                    }
                }

                let event = self.process_key(&e.key, &e.code, &e.modifiers, true, true, true);
                if event.contains(TextEvent::TEXT_CHANGED) {
                    self.run_parser();
                    self.dragging = TextDragging::None;
                }

                if !event.is_empty() {
                    processed = true;
                }

                None
            }
            EditableEvent::KeyUp(e) => {
                if e.code == Code::ShiftLeft {
                    if let TextDragging::FromCursorToPoint { shift, .. } = &mut self.dragging {
                        *shift = false;
                    }
                } else {
                    self.dragging = TextDragging::None;
                }

                None
            }
        };

        if let Some((cursor_id, cursor_position, cursor_selection)) = res {
            if self.dragging.has_cursor_coords() {
                UsePlatform::current()
                    .send(EventLoopMessage::RemeasureTextGroup(TextGroupMeasurement {
                        text_id: self.text_id,
                        cursor_id,
                        cursor_position,
                        cursor_selection,
                    }))
                    .unwrap();
            }
        }

        processed
    }
}

impl Display for EditorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.rope.borrow().to_string())
    }
}

impl TextEditor for EditorData {
    type LinesIterator<'a>
        = LinesIterator<'a>
    where
        Self: 'a;

    fn lines(&self) -> Self::LinesIterator<'_> {
        unimplemented!("Unused.")
    }

    fn insert_char(&mut self, ch: char, idx: usize) -> usize {
        let idx_utf8 = self.utf16_cu_to_char(idx);

        let len_before_insert = self.len_utf16_cu();
        self.rope.borrow_mut().insert_char(idx_utf8, ch);
        let len_after_insert = self.len_utf16_cu();

        let inserted_text_len = len_after_insert - len_before_insert;

        self.history.push_change(HistoryChange::InsertChar {
            idx,
            ch,
            len: inserted_text_len,
        });

        inserted_text_len
    }

    fn insert(&mut self, text: &str, idx: usize) -> usize {
        let idx_utf8 = self.utf16_cu_to_char(idx);

        let len_before_insert = self.len_utf16_cu();
        self.rope.borrow_mut().insert(idx_utf8, text);
        let len_after_insert = self.len_utf16_cu();

        let inserted_text_len = len_after_insert - len_before_insert;

        self.history.push_change(HistoryChange::InsertText {
            idx,
            text: text.to_owned(),
            len: inserted_text_len,
        });

        inserted_text_len
    }

    fn remove(&mut self, range_utf16: Range<usize>) -> usize {
        let range =
            self.utf16_cu_to_char(range_utf16.start)..self.utf16_cu_to_char(range_utf16.end);
        let text = self.rope.borrow().slice(range.clone()).to_string();

        let len_before_remove = self.len_utf16_cu();
        self.rope.borrow_mut().remove(range);
        let len_after_remove = self.len_utf16_cu();

        let removed_text_len = len_before_remove - len_after_remove;

        self.history.push_change(HistoryChange::Remove {
            idx: range_utf16.end - removed_text_len,
            text,
            len: removed_text_len,
        });

        removed_text_len
    }

    fn char_to_line(&self, char_idx: usize) -> usize {
        self.rope.borrow().char_to_line(char_idx)
    }

    fn line_to_char(&self, line_idx: usize) -> usize {
        self.rope.borrow().line_to_char(line_idx)
    }

    fn utf16_cu_to_char(&self, utf16_cu_idx: usize) -> usize {
        self.rope.borrow().utf16_cu_to_char(utf16_cu_idx)
    }

    fn char_to_utf16_cu(&self, idx: usize) -> usize {
        self.rope.borrow().char_to_utf16_cu(idx)
    }

    fn line(&self, line_idx: usize) -> Option<Line<'_>> {
        let rope = self.rope.borrow();
        let line = rope.get_line(line_idx);

        line.map(|line| Line {
            text: Cow::Owned(line.to_string()),
            utf16_len: line.len_utf16_cu(),
        })
    }

    fn len_lines(&self) -> usize {
        self.rope.borrow().len_lines()
    }

    fn len_chars(&self) -> usize {
        self.rope.borrow().len_chars()
    }

    fn len_utf16_cu(&self) -> usize {
        self.rope.borrow().len_utf16_cu()
    }

    fn cursor(&self) -> &TextCursor {
        &self.cursor
    }

    fn cursor_mut(&mut self) -> &mut TextCursor {
        &mut self.cursor
    }

    fn expand_selection_to_cursor(&mut self) {
        let pos = self.cursor_pos();
        if let Some(selected) = self.selected.as_mut() {
            selected.1 = pos;
        } else {
            self.selected = Some((self.cursor_pos(), self.cursor_pos()))
        }
    }

    fn has_any_selection(&self) -> bool {
        self.selected.is_some()
    }

    fn get_selection(&self) -> Option<(usize, usize)> {
        self.selected
    }

    fn get_visible_selection(&self, editor_id: usize) -> Option<(usize, usize)> {
        let (selected_from, selected_to) = self.selected?;
        let selected_from_row = self.char_to_line(self.utf16_cu_to_char(selected_from));
        let selected_to_row = self.char_to_line(self.utf16_cu_to_char(selected_to));

        let editor_row_idx = self.char_to_utf16_cu(self.line_to_char(editor_id));
        let selected_from_row_idx = self.char_to_utf16_cu(self.line_to_char(selected_from_row));
        let selected_to_row_idx = self.char_to_utf16_cu(self.line_to_char(selected_to_row));

        let selected_from_col_idx = selected_from - selected_from_row_idx;
        let selected_to_col_idx = selected_to - selected_to_row_idx;

        // Between starting line and endling line
        if (editor_id > selected_from_row && editor_id < selected_to_row)
            || (editor_id < selected_from_row && editor_id > selected_to_row)
        {
            let len = self.line(editor_id).unwrap().utf16_len();
            return Some((0, len));
        }

        let highlights = match selected_from_row.cmp(&selected_to_row) {
            // Selection direction is from bottom -> top
            Ordering::Greater => {
                if selected_from_row == editor_id {
                    // Starting line
                    Some((0, selected_from_col_idx))
                } else if selected_to_row == editor_id {
                    // Ending line
                    let len = self.line(selected_to_row).unwrap().utf16_len();
                    Some((selected_to_col_idx, len))
                } else {
                    None
                }
            }
            // Selection direction is from top -> bottom
            Ordering::Less => {
                if selected_from_row == editor_id {
                    // Starting line
                    let len = self.line(selected_from_row).unwrap().utf16_len();
                    Some((selected_from_col_idx, len))
                } else if selected_to_row == editor_id {
                    // Ending line
                    Some((0, selected_to_col_idx))
                } else {
                    None
                }
            }
            Ordering::Equal if selected_from_row == editor_id => {
                // Starting and endline line are the same
                Some((selected_from - editor_row_idx, selected_to - editor_row_idx))
            }
            _ => None,
        };

        highlights
    }

    fn set(&mut self, text: &str) {
        self.rope.borrow_mut().remove(0..);
        self.rope.borrow_mut().insert(0, text);
    }

    fn clear_selection(&mut self) {
        self.selected = None;
    }

    fn measure_new_selection(&self, from: usize, to: usize, editor_id: usize) -> (usize, usize) {
        let row_idx = self.line_to_char(editor_id);
        let row_idx = self.char_to_utf16_cu(row_idx);
        if let Some((start, _)) = self.selected {
            (start, row_idx + to)
        } else {
            (row_idx + from, row_idx + to)
        }
    }

    fn measure_new_cursor(&self, to: usize, editor_id: usize) -> TextCursor {
        let row_char = self.line_to_char(editor_id);
        let pos = self.char_to_utf16_cu(row_char) + to;
        TextCursor::new(pos)
    }

    fn get_clipboard(&mut self) -> &mut UseClipboard {
        &mut self.clipboard
    }

    fn set_selection(&mut self, selected: (usize, usize)) {
        self.selected = Some(selected);
    }

    fn get_selected_text(&self) -> Option<String> {
        let (start, end) = self.get_selection_range()?;

        Some(self.rope.borrow().get_slice(start..end)?.to_string())
    }

    fn get_selection_range(&self) -> Option<(usize, usize)> {
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
            self.history.redo(&mut self.rope.borrow_mut())
        } else {
            None
        }
    }

    fn undo(&mut self) -> Option<usize> {
        if self.history.can_undo() {
            self.history.undo(&mut self.rope.borrow_mut())
        } else {
            None
        }
    }

    fn editor_history(&mut self) -> &mut EditorHistory {
        &mut self.history
    }

    fn get_identation(&self) -> u8 {
        4
    }
}
