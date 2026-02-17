use std::{
    borrow::Cow,
    cell::RefCell,
    cmp::Ordering,
    fmt::Display,
    ops::{Mul, Range},
    path::PathBuf,
    rc::Rc,
    time::Duration,
};

use freya::{elements::paragraph::ParagraphHolderInner, prelude::*, text_edit::*};
use ropey::Rope;
use skia_safe::textlayout::FontCollection;
use tree_sitter::InputEdit;

use crate::{fs::FSTransport, languages::LanguageId, metrics::EditorMetrics, syntax::InputEditExt};

pub type SharedRope = Rc<RefCell<Rope>>;

#[derive(Clone, PartialEq, Debug)]
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
    pub fn content_id(&self) -> Option<String> {
        match self {
            Self::Memory { .. } => None,
            Self::FS { path, .. } => Some(path.file_name().unwrap().to_str().unwrap().to_owned()),
        }
    }

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

pub struct TabEditorData {
    pub(crate) data: EditorData,
    pub(crate) transport: FSTransport,
}

pub struct EditorData {
    pub(crate) editor_type: EditorType,
    pub(crate) history: EditorHistory,
    pub(crate) rope: SharedRope,
    pub(crate) selection: TextSelection,
    pub(crate) last_saved_history_change: usize,
    pub(crate) metrics: EditorMetrics,
    pub(crate) dragging: TextDragging,
    pub(crate) scrolls: (i32, i32),
    pending_edit: Option<InputEdit>,
}

impl EditorData {
    pub fn new(editor_type: EditorType, rope: SharedRope) -> Self {
        Self {
            editor_type,
            rope,
            selection: TextSelection::new_cursor(0),
            history: EditorHistory::new(Duration::from_secs(1)),
            last_saved_history_change: 0,
            metrics: EditorMetrics::new(),
            dragging: TextDragging::default(),
            scrolls: (0, 0),
            pending_edit: None,
        }
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

    pub fn run_parser(&mut self) {
        let language_id = self.editor_type.language_id();
        let edit = self.pending_edit.take();
        self.metrics
            .run_parser(&self.rope.borrow(), language_id, edit);
    }

    pub fn measure_longest_line(&mut self, font_size: f32, font_collection: &FontCollection) {
        self.metrics
            .measure_longest_line(font_size, &self.rope.borrow(), font_collection);
    }

    pub fn editor_type(&self) -> &EditorType {
        &self.editor_type
    }

    pub fn process(&mut self, edit_event: EditableEvent) -> bool {
        let mut processed = false;
        match edit_event {
            EditableEvent::Down {
                location,
                editor_line,
                holder,
            } => {
                let holder = holder.0.borrow();
                let ParagraphHolderInner {
                    paragraph,
                    scale_factor,
                } = holder.as_ref().unwrap();

                let current_selection = self.selection().clone();

                if self.dragging.shift || self.dragging.clicked {
                    self.selection_mut().set_as_range();
                } else {
                    self.clear_selection();
                }

                if &current_selection != self.selection() {
                    processed = true;
                }

                self.dragging.clicked = true;

                let char_position = paragraph.get_glyph_position_at_coordinate(
                    location.mul(*scale_factor).to_i32().to_tuple(),
                );
                let press_selection =
                    self.measure_selection(char_position.position as usize, editor_line);

                let new_selection = match EventsCombos::pressed(location) {
                    PressEventType::Triple => {
                        let line = self.char_to_line(press_selection.pos());
                        let line_char = self.line_to_char(line);
                        let line_len = self.line(line).unwrap().utf16_len();
                        TextSelection::new_range((line_char, line_char + line_len))
                    }
                    PressEventType::Double => {
                        let range = self.find_word_boundaries(press_selection.pos());
                        TextSelection::new_range(range)
                    }
                    PressEventType::Single => press_selection,
                };

                if *self.selection() != new_selection {
                    *self.selection_mut() = new_selection;
                    processed = true;
                }
            }
            EditableEvent::Move {
                location,
                editor_line,
                holder,
            } => {
                if self.dragging.clicked {
                    let paragraph = holder.0.borrow();
                    let ParagraphHolderInner {
                        paragraph,
                        scale_factor,
                    } = paragraph.as_ref().unwrap();

                    let dist_position = location.mul(*scale_factor);

                    // Calculate the end of the highlighting
                    let dist_char = paragraph
                        .get_glyph_position_at_coordinate(dist_position.to_i32().to_tuple());
                    let to = dist_char.position as usize;

                    if self.get_selection().is_none() {
                        self.selection_mut().set_as_range();
                        processed = true;
                    }

                    let current_selection = self.selection().clone();

                    let new_selection = self.measure_selection(to, editor_line);

                    // Update the cursor if it has changed
                    if current_selection != new_selection {
                        *self.selection_mut() = new_selection;
                        processed = true;
                    }
                }
            }
            EditableEvent::Release => {
                self.dragging.clicked = false;
            }
            EditableEvent::KeyDown { key, modifiers } => {
                match key {
                    // Handle dragging
                    Key::Named(NamedKey::Shift) => {
                        self.dragging.shift = true;
                    }
                    // Handle editing
                    _ => {
                        let event = self.process_key(key, &modifiers, true, true, true);
                        if event.contains(TextEvent::TEXT_CHANGED) {
                            self.run_parser();
                            self.dragging = TextDragging::default();
                        }
                        if !event.is_empty() {
                            processed = true;
                        }
                    }
                }
            }
            EditableEvent::KeyUp { key, .. } => {
                if *key == Key::Named(NamedKey::Shift) {
                    self.dragging.shift = false;
                }
            }
        };
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
        let selection = self.selection.clone();

        // Capture byte offset and position before mutation for InputEdit.
        let mut rope = self.rope.borrow_mut();
        let start_byte = rope.char_to_byte(idx_utf8);
        let start_line = rope.char_to_line(idx_utf8);
        let start_line_byte = rope.line_to_byte(start_line);
        let start_col = start_byte - start_line_byte;

        let len_before_insert = rope.len_utf16_cu();
        rope.insert_char(idx_utf8, ch);
        let len_after_insert = rope.len_utf16_cu();

        let inserted_text_len = len_after_insert - len_before_insert;

        // Compute new end position after insertion.
        let new_end_char = idx_utf8 + 1; // one char inserted
        let new_end_byte = rope.char_to_byte(new_end_char);
        let new_end_line = rope.char_to_line(new_end_char);
        let new_end_line_byte = rope.line_to_byte(new_end_line);
        let new_end_col = new_end_byte - new_end_line_byte;

        self.pending_edit = Some(InputEdit::new(
            start_byte,
            start_byte,
            new_end_byte,
            (start_line, start_col),
            (start_line, start_col),
            (new_end_line, new_end_col),
        ));

        self.history.push_change(HistoryChange::InsertChar {
            idx,
            ch,
            len: inserted_text_len,
            selection,
        });

        inserted_text_len
    }

    fn insert(&mut self, text: &str, idx: usize) -> usize {
        let idx_utf8 = self.utf16_cu_to_char(idx);
        let selection = self.selection.clone();

        // Capture byte offset and position before mutation for InputEdit.
        let mut rope = self.rope.borrow_mut();
        let start_byte = rope.char_to_byte(idx_utf8);
        let start_line = rope.char_to_line(idx_utf8);
        let start_line_byte = rope.line_to_byte(start_line);
        let start_col = start_byte - start_line_byte;

        let len_before_insert = rope.len_utf16_cu();
        rope.insert(idx_utf8, text);
        let len_after_insert = rope.len_utf16_cu();

        let inserted_text_len = len_after_insert - len_before_insert;

        // Compute new end position after insertion.
        let inserted_chars = text.chars().count();
        let new_end_char = idx_utf8 + inserted_chars;
        let new_end_byte = rope.char_to_byte(new_end_char);
        let new_end_line = rope.char_to_line(new_end_char);
        let new_end_line_byte = rope.line_to_byte(new_end_line);
        let new_end_col = new_end_byte - new_end_line_byte;

        self.pending_edit = Some(InputEdit::new(
            start_byte,
            start_byte,
            new_end_byte,
            (start_line, start_col),
            (start_line, start_col),
            (new_end_line, new_end_col),
        ));

        self.history.push_change(HistoryChange::InsertText {
            idx,
            text: text.to_owned(),
            len: inserted_text_len,
            selection,
        });

        inserted_text_len
    }

    fn remove(&mut self, range_utf16: Range<usize>) -> usize {
        let range =
            self.utf16_cu_to_char(range_utf16.start)..self.utf16_cu_to_char(range_utf16.end);
        let text = self.rope.borrow().slice(range.clone()).to_string();
        let selection = self.selection.clone();

        // Capture byte offsets and positions before mutation for InputEdit.
        let mut rope = self.rope.borrow_mut();
        let start_byte = rope.char_to_byte(range.start);
        let old_end_byte = rope.char_to_byte(range.end);
        let start_line = rope.char_to_line(range.start);
        let start_line_byte = rope.line_to_byte(start_line);
        let start_col = start_byte - start_line_byte;
        let old_end_line = rope.char_to_line(range.end);
        let old_end_line_byte = rope.line_to_byte(old_end_line);
        let old_end_col = old_end_byte - old_end_line_byte;

        let len_before_remove = rope.len_utf16_cu();
        rope.remove(range);
        let len_after_remove = rope.len_utf16_cu();

        let removed_text_len = len_before_remove - len_after_remove;

        // After removal, new_end == start (the removed range collapses to a point).
        self.pending_edit = Some(InputEdit::new(
            start_byte,
            old_end_byte,
            start_byte,
            (start_line, start_col),
            (old_end_line, old_end_col),
            (start_line, start_col),
        ));

        self.history.push_change(HistoryChange::Remove {
            idx: range_utf16.end - removed_text_len,
            text,
            len: removed_text_len,
            selection,
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

    fn has_any_selection(&self) -> bool {
        self.selection.is_range()
    }

    fn get_selection(&self) -> Option<(usize, usize)> {
        match self.selection {
            TextSelection::Cursor(_) => None,
            TextSelection::Range { from, to } => Some((from, to)),
        }
    }

    fn get_visible_selection(&self, editor_id: EditorLine) -> Option<(usize, usize)> {
        let editor_id = match editor_id {
            EditorLine::Paragraph(editor_id) => editor_id,
            _ => unreachable!(),
        };
        let (selected_from, selected_to) = self.get_selection()?;
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

        match selected_from_row.cmp(&selected_to_row) {
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
        }
    }

    fn set(&mut self, text: &str) {
        self.rope.borrow_mut().remove(0..);
        self.rope.borrow_mut().insert(0, text);
    }

    fn clear_selection(&mut self) {
        let end = self.selection().end();
        self.selection_mut().set_as_cursor();
        self.selection_mut().move_to(end);
    }

    fn set_selection(&mut self, (from, to): (usize, usize)) {
        self.selection = TextSelection::Range { from, to };
    }

    fn get_selected_text(&self) -> Option<String> {
        let (start, end) = self.get_selection_range()?;

        Some(self.rope.borrow().get_slice(start..end)?.to_string())
    }

    fn get_selection_range(&self) -> Option<(usize, usize)> {
        let (start, end) = match self.selection {
            TextSelection::Cursor(_) => return None,
            TextSelection::Range { from, to } => (from, to),
        };

        // Use left-to-right selection
        let (start, end) = if start < end {
            (start, end)
        } else {
            (end, start)
        };

        Some((start, end))
    }

    fn undo(&mut self) -> Option<TextSelection> {
        // Undo can make arbitrary changes — invalidate the tree for a full re-parse.
        self.pending_edit = None;
        self.metrics.highlighter.invalidate_tree();
        self.history.undo(&mut self.rope.borrow_mut())
    }

    fn redo(&mut self) -> Option<TextSelection> {
        // Redo can make arbitrary changes — invalidate the tree for a full re-parse.
        self.pending_edit = None;
        self.metrics.highlighter.invalidate_tree();
        self.history.redo(&mut self.rope.borrow_mut())
    }

    fn editor_history(&mut self) -> &mut EditorHistory {
        &mut self.history
    }

    fn selection(&self) -> &TextSelection {
        &self.selection
    }

    fn selection_mut(&mut self) -> &mut TextSelection {
        &mut self.selection
    }

    fn get_indentation(&self) -> u8 {
        4
    }
}
