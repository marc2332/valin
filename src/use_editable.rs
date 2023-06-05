use dioxus::core::AttributeValue;
use freya::prelude::{keyboard::Key, *};
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
use tokio::sync::{mpsc::unbounded_channel, mpsc::UnboundedSender};
use torin::geometry::CursorPoint;
use uuid::Uuid;
use winit::event_loop::EventLoopProxy;

#[derive(Clone, Default)]
pub struct Panel {
    active_editor: Option<usize>,
    editors: Vec<EditorData>,
}

impl Panel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active_editor(&self) -> Option<usize> {
        self.active_editor
    }

    pub fn editor(&self, editor: usize) -> &EditorData {
        &self.editors[editor]
    }

    pub fn editors(&self) -> &[EditorData] {
        &self.editors
    }

    pub fn set_active_editor(&mut self, active_editor: usize) {
        self.active_editor = Some(active_editor);
    }
}

#[derive(Clone)]
pub struct EditorManager {
    is_focused: bool,
    focused_panel: usize,
    panes: Vec<Panel>,
    font_size: f32,
    line_height: f32,
}

impl Default for EditorManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorManager {
    pub fn new() -> Self {
        Self {
            is_focused: true,
            focused_panel: 0,
            panes: vec![Panel::new()],
            font_size: 17.0,
            line_height: 1.3,
        }
    }

    pub fn set_fontsize(&mut self, fontsize: f32) {
        self.font_size = fontsize;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    pub fn line_height(&self) -> f32 {
        self.line_height
    }

    pub fn focused_panel(&self) -> usize {
        self.focused_panel
    }

    pub fn push_editor(&mut self, editor: EditorData, panel: usize, focus: bool) {
        self.panes[panel].editors.push(editor);

        if focus {
            self.focused_panel = panel;
            self.panes[panel].active_editor = Some(self.panes[panel].editors.len() - 1);
        }
    }

    pub fn push_panel(&mut self, panel: Panel) {
        self.panes.push(panel);
    }

    pub fn panels(&self) -> &[Panel] {
        &self.panes
    }

    pub fn panel(&self, panel: usize) -> &Panel {
        &self.panes[panel]
    }

    pub fn panel_mut(&mut self, panel: usize) -> &mut Panel {
        &mut self.panes[panel]
    }

    pub fn set_focused_panel(&mut self, panel: usize) {
        self.focused_panel = panel;
    }

    pub fn close_panel(&mut self, panel: usize) {
        if self.panes.len() > 1 {
            self.panes.remove(panel);
            if self.focused_panel > 0 {
                self.focused_panel -= 1;
            }
        }
    }
}

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
    rope: Rope,
    path: PathBuf,

    /// Selected text range
    selected: Option<(usize, usize)>,
}

impl EditorData {
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

impl EditorData {
    pub fn new(path: PathBuf, rope: Rope, (row, col): (usize, usize)) -> Self {
        Self {
            path,
            rope,
            cursor: TextCursor::new(row, col),
            selected: None,
        }
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
        self.rope.insert_char(char_idx, char);
    }

    fn insert(&mut self, text: &str, char_idx: usize) {
        self.rope.insert(char_idx, text);
    }

    fn remove(&mut self, range: Range<usize>) {
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

pub type EditorState = UseState<EditorManager>;

/// Manage an editable content.
#[derive(Clone)]
pub struct UseEditable {
    pub(crate) editor: EditorState,
    pub(crate) cursor_reference: CursorReference,
    pub(crate) selecting_text_with_mouse: UseRef<Option<CursorPoint>>,
    pub(crate) event_loop_proxy: Option<EventLoopProxy<EventMessage>>,
    pub(crate) pane_index: usize,
    pub(crate) editor_index: usize,
    pub(crate) highlight_trigger: UnboundedSender<()>,
}

impl UseEditable {
    /// Create a cursor attribute.
    pub fn cursor_attr<'a, T>(&self, cx: Scope<'a, T>) -> AttributeValue<'a> {
        cx.any_value(CustomAttributeValues::CursorReference(
            self.cursor_reference.clone(),
        ))
    }

    /// Create a highlights attribute.
    pub fn highlights_attr<'a, T>(&self, cx: Scope<'a, T>, editor_id: usize) -> AttributeValue<'a> {
        let editor = &self.editor.panes[self.pane_index].editors[self.editor_index];
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

                self.editor.with_mut(|text_editor| {
                    let editor = &mut text_editor.panes[self.pane_index].editors[self.editor_index];
                    editor.unhighlight();
                });
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
                if e.key == Key::Escape {
                    return;
                }
                self.editor.with_mut(|text_editor| {
                    let editor = &mut text_editor.panes[self.pane_index].editors[self.editor_index];
                    let event = editor.process_key(&e.key, &e.code, &e.modifiers);
                    if event == TextEvent::TextChanged {
                        self.highlight_trigger.send(()).ok();
                        *self.selecting_text_with_mouse.write_silent() = None;
                    }
                });
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
    text_editor: &UseState<EditorManager>,
    pane_index: usize,
    editor_index: usize,
    highlight_trigger: UnboundedSender<()>,
) -> UseEditable {
    let id = cx.use_hook(Uuid::new_v4);
    let event_loop_proxy = cx.consume_context::<EventLoopProxy<EventMessage>>();

    let cursor_channels = cx.use_hook(|| {
        let (tx, rx) = unbounded_channel::<CursorLayoutResponse>();
        (tx, Some(rx))
    });

    let cursor_reference = cx.use_hook(|| CursorReference {
        text_id: *id,
        agent: cursor_channels.0.clone(),
        cursor_position: Arc::new(Mutex::new(None)),
        cursor_id: Arc::new(Mutex::new(None)),
        cursor_selections: Arc::new(Mutex::new(None)),
    });

    let selecting_text_with_mouse = use_ref(cx, || None);

    let use_editable = UseEditable {
        editor: text_editor.clone(),
        cursor_reference: cursor_reference.clone(),
        selecting_text_with_mouse: selecting_text_with_mouse.clone(),
        event_loop_proxy,
        pane_index,
        editor_index,
        highlight_trigger,
    };

    // Listen for new calculations from the layout engine
    use_effect(cx, (), move |_| {
        let cursor_reference = cursor_reference.clone();
        let cursor_receiver = cursor_channels.1.take();
        let text_editor = text_editor.clone();

        async move {
            let mut cursor_receiver = cursor_receiver.unwrap();

            while let Some(message) = cursor_receiver.recv().await {
                match message {
                    // Update the cursor position calculated by the layout
                    CursorLayoutResponse::CursorPosition { position, id } => {
                        let editor = &text_editor.current().panes[pane_index].editors[editor_index];

                        let new_current_line = editor.rope.line(id);

                        // Use the line lenght as new column if the clicked column surpases the length
                        let new_cursor = if position >= new_current_line.chars().len() {
                            (new_current_line.chars().len(), id)
                        } else {
                            (position, id)
                        };

                        // Only update if it's actually different
                        if editor.cursor.as_tuple() != new_cursor {
                            text_editor.with_mut(|text_editor| {
                                let editor =
                                    &mut text_editor.panes[pane_index].editors[editor_index];
                                editor.cursor.set_col(new_cursor.0);
                                editor.cursor.set_row(new_cursor.1);
                                editor.unhighlight();
                            });
                        }

                        // Remove the current calcutions so the layout engine doesn't try to calculate again
                        cursor_reference.set_cursor_position(None);
                    }
                    // Update the text selections calculated by the layout
                    CursorLayoutResponse::TextSelection { from, to, id } => {
                        text_editor.with_mut(|text_editor| {
                            let editor = &mut text_editor.panes[pane_index].editors[editor_index];
                            editor.highlight_text(from, to, id);
                        });
                        cursor_reference.set_cursor_selections(None);
                    }
                }
            }
        }
    });

    use_editable
}
