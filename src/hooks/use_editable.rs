use dioxus::core::AttributeValue;
use freya::prelude::{
    keyboard::{Code, Key, Modifiers},
    *,
};
use freya_common::{CursorLayoutResponse, EventMessage};
use freya_node_state::CursorReference;
use ropey::iter::Lines;
use std::{
    cmp::Ordering,
    fmt::Display,
    ops::Range,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use torin::geometry::CursorPoint;
use uuid::Uuid;
use winit::event_loop::EventLoopProxy;

use crate::{
    history::{History, HistoryChange},
    hooks::UseManager,
    lsp::LanguageId,
};

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

#[derive(Clone)]
pub struct EditorData {
    cursor: TextCursor,
    history: History,
    rope: Rope,
    path: PathBuf,
    pub root_path: PathBuf,
    pub language_id: LanguageId,

    /// Selected text range
    selected: Option<(usize, usize)>,
}

impl EditorData {
    pub fn new(path: PathBuf, rope: Rope, (row, col): (usize, usize), root_path: PathBuf) -> Self {
        let language_id = if let Some(ext) = path.extension() {
            LanguageId::parse(ext.to_str().unwrap())
        } else {
            LanguageId::default()
        };
        Self {
            path,
            rope,
            cursor: TextCursor::new(row, col),
            selected: None,
            language_id,
            root_path,
            history: History::new(),
        }
    }

    pub fn move_cursor_to_idx(&mut self, idx: usize) {
        let row = self.rope.byte_to_line(idx);
        let line_idx = self.rope.line_to_byte(row);
        let col = idx - line_idx;
        self.cursor_mut().move_to(row, col);
    }

    pub fn redo(&mut self) {
        if self.history.can_redo() {
            let cursor_idx = self.history.redo(&mut self.rope);
            if let Some(cursor_idx) = cursor_idx {
                self.move_cursor_to_idx(cursor_idx);
            }
        }
    }

    pub fn undo(&mut self) {
        if self.history.can_undo() {
            let cursor_idx = self.history.undo(&mut self.rope);
            if let Some(cursor_idx) = cursor_idx {
                self.move_cursor_to_idx(cursor_idx);
            }
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
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
            ch: char,
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
}

/// Manage an editable content.
#[derive(Clone)]
pub struct UseEdit {
    pub(crate) manager: UseManager,
    pub(crate) cursor_reference: CursorReference,
    pub(crate) selecting_text_with_mouse: UseRef<Option<CursorPoint>>,
    pub(crate) event_loop_proxy: Option<EventLoopProxy<EventMessage>>,
    pub(crate) pane_index: usize,
    pub(crate) editor_index: usize,
    pub(crate) coroutine_coroutine: UnboundedSender<()>,
}

impl UseEdit {
    /// Create a cursor attribute.
    pub fn cursor_attr<'a, T>(&self, cx: Scope<'a, T>) -> AttributeValue<'a> {
        cx.any_value(CustomAttributeValues::CursorReference(
            self.cursor_reference.clone(),
        ))
    }

    /// Create a highlights attribute.
    pub fn highlights_attr<'a, T>(&self, cx: Scope<'a, T>, editor_id: usize) -> AttributeValue<'a> {
        let manager = self.manager.current();
        let editor = manager
            .panel(self.pane_index)
            .tab(self.editor_index)
            .as_text_editor()
            .unwrap();
        cx.any_value(CustomAttributeValues::TextHighlights(
            editor
                .highlights(editor_id)
                .map(|v| vec![v])
                .unwrap_or_default(),
        ))
    }

    /// Process a [`EditableEvent`] event.
    pub fn process_event(&self, edit_event: &EditableEvent) {
        match edit_event {
            EditableEvent::MouseDown(e, id) => {
                let coords = e.get_element_coordinates();
                *self.selecting_text_with_mouse.write_silent() = Some(coords);

                self.cursor_reference.set_id(Some(*id));
                self.cursor_reference.set_cursor_position(Some(coords));
                let mut manager = self.manager.write();
                let editor = manager
                    .panel_mut(self.pane_index)
                    .tab_mut(self.editor_index)
                    .as_text_editor_mut()
                    .unwrap();
                editor.unhighlight();
            }
            EditableEvent::MouseOver(e, id) => {
                self.selecting_text_with_mouse.with(|selecting_text| {
                    if let Some(current_dragging) = selecting_text {
                        let coords = e.get_element_coordinates();

                        self.cursor_reference.set_id(Some(*id));
                        self.cursor_reference
                            .set_cursor_selections(Some((*current_dragging, coords)));
                    }
                });
            }
            EditableEvent::Click => {
                *self.selecting_text_with_mouse.write_silent() = None;
            }
            EditableEvent::KeyDown(e) => {
                let is_plus = e.key == Key::Character("+".to_string());
                let is_minus = e.key == Key::Character("-".to_string());
                let is_e = e.key == Key::Character("e".to_string());

                if e.key == Key::Escape
                    || (e.modifiers.contains(Modifiers::ALT) && (is_plus || is_minus || is_e))
                {
                    return;
                }

                let mut manager = self.manager.write();
                let editor = manager
                    .panel_mut(self.pane_index)
                    .tab_mut(self.editor_index)
                    .as_text_editor_mut()
                    .unwrap();

                if e.modifiers.contains(Modifiers::CONTROL) {
                    if e.code == Code::KeyZ {
                        editor.undo();
                        self.coroutine_coroutine.send(()).unwrap();
                        return;
                    } else if e.code == Code::KeyY {
                        editor.redo();
                        self.coroutine_coroutine.send(()).unwrap();
                        return;
                    }
                }

                let event = editor.process_key(&e.key, &e.code, &e.modifiers);
                if event == TextEvent::TextChanged {
                    self.coroutine_coroutine.send(()).unwrap();
                    *self.selecting_text_with_mouse.write_silent() = None;
                }
            }
        }

        if self.selecting_text_with_mouse.read().is_some() {
            if let Some(event_loop_proxy) = &self.event_loop_proxy {
                event_loop_proxy
                    .send_event(EventMessage::RemeasureTextGroup(
                        self.cursor_reference.text_id,
                    ))
                    .unwrap();
            }
        }
    }
}

pub fn use_edit(
    cx: &ScopeState,
    manager: &UseManager,
    pane_index: usize,
    editor_index: usize,
    coroutine_coroutine: &UnboundedSender<()>,
) -> UseEdit {
    let event_loop_proxy = cx.consume_context::<EventLoopProxy<EventMessage>>();
    let selecting_text_with_mouse = use_ref(cx, || None);

    let cursor_reference = use_memo(cx, &(pane_index, editor_index), |_| {
        let text_id = Uuid::new_v4();
        let (cursor_sender, mut cursor_receiver) = unbounded_channel::<CursorLayoutResponse>();

        let cursor_reference = CursorReference {
            text_id,
            agent: cursor_sender.clone(),
            cursor_position: Arc::new(Mutex::new(None)),
            cursor_id: Arc::new(Mutex::new(None)),
            cursor_selections: Arc::new(Mutex::new(None)),
        };

        cx.spawn({
            to_owned![manager, cursor_reference];
            async move {
                while let Some(message) = cursor_receiver.recv().await {
                    match message {
                        // Update the cursor position calculated by the layout
                        CursorLayoutResponse::CursorPosition { position, id } => {
                            let mut manager = manager.write();
                            let editor = manager
                                .panel(pane_index)
                                .tab(editor_index)
                                .as_text_editor()
                                .unwrap();

                            let new_current_line = editor.rope.line(id);

                            // Use the line lenght as new column if the clicked column surpases the length
                            let new_cursor = if position >= new_current_line.chars().len() {
                                (new_current_line.chars().len(), id)
                            } else {
                                (position, id)
                            };

                            // Only update if it's actually different
                            if editor.cursor.as_tuple() != new_cursor {
                                let editor = manager
                                    .panel_mut(pane_index)
                                    .tab_mut(editor_index)
                                    .as_text_editor_mut()
                                    .unwrap();
                                editor.cursor.set_col(new_cursor.0);
                                editor.cursor.set_row(new_cursor.1);
                                editor.unhighlight();
                            }

                            // Remove the current calcutions so the layout engine doesn't try to calculate again
                            cursor_reference.set_cursor_position(None);
                        }
                        // Update the text selections calculated by the layout
                        CursorLayoutResponse::TextSelection { from, to, id } => {
                            let mut manager = manager.write();
                            let editor = manager
                                .panel_mut(pane_index)
                                .tab_mut(editor_index)
                                .as_text_editor_mut()
                                .unwrap();
                            editor.highlight_text(from, to, id);
                            cursor_reference.set_cursor_selections(None);
                        }
                    }
                }
            }
        });

        cursor_reference
    });

    UseEdit {
        manager: manager.clone(),
        cursor_reference: cursor_reference.clone(),
        selecting_text_with_mouse: selecting_text_with_mouse.clone(),
        event_loop_proxy,
        pane_index,
        editor_index,
        coroutine_coroutine: coroutine_coroutine.clone(),
    }
}
