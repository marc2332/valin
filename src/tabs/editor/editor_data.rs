use std::{cmp::Ordering, fmt::Display, ops::Range, path::PathBuf};

use dioxus_sdk::clipboard::UseClipboard;
use freya::hooks::{EditorHistory, HistoryChange, Line, TextCursor, TextEditor};
use freya::prelude::Rope;
use freya_hooks::LinesIterator;
use lsp_types::Url;
use skia_safe::textlayout::FontCollection;

use crate::{fs::FSTransport, lsp::LanguageId, metrics::EditorMetrics};

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

pub struct EditorData {
    pub(crate) editor_type: EditorType,
    pub(crate) cursor: TextCursor,
    pub(crate) history: EditorHistory,
    pub(crate) rope: Rope,
    pub(crate) selected: Option<(usize, usize)>,
    pub(crate) clipboard: UseClipboard,
    pub(crate) last_saved_history_change: usize,
    pub(crate) transport: FSTransport,
    pub(crate) metrics: EditorMetrics,
}

impl EditorData {
    pub fn new(
        editor_type: EditorType,
        rope: Rope,
        (row, col): (usize, usize),
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
            rope,
            cursor: TextCursor::new(row, col),
            selected: None,
            history: EditorHistory::new(),
            last_saved_history_change: 0,
            clipboard,
            transport,
            metrics,
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

    pub fn run_parser(&mut self) {
        self.metrics.run_parser(&self.rope);
    }

    pub fn measure_longest_line(&mut self, font_size: f32, font_collection: &FontCollection) {
        self.metrics
            .measure_longest_line(font_size, &self.rope, font_collection);
    }

    pub fn editor_type(&self) -> &EditorType {
        &self.editor_type
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

    fn utf16_cu_to_char(&self, utf16_cu_idx: usize) -> usize {
        self.rope.utf16_cu_to_char(utf16_cu_idx)
    }

    fn char_to_utf16_cu(&self, idx: usize) -> usize {
        self.rope.char_to_utf16_cu(idx)
    }

    fn line(&self, line_idx: usize) -> Option<Line<'_>> {
        let line = self.rope.get_line(line_idx);

        line.map(|line| Line { text: line.into() })
    }

    fn len_lines<'a>(&self) -> usize {
        self.rope.len_lines()
    }

    fn len_chars(&self) -> usize {
        self.rope.len_chars()
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
            self.history.redo(&mut self.rope)
        } else {
            None
        }
    }

    fn undo(&mut self) -> Option<usize> {
        if self.history.can_undo() {
            self.history.undo(&mut self.rope)
        } else {
            None
        }
    }
}
