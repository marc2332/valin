use freya::prelude::*;
use freya_node_state::CursorReference;
use ropey::iter::Lines;
use std::{
    fmt::Display,
    ops::Range,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Mutex},
};
use tokio::sync::{mpsc::unbounded_channel, mpsc::UnboundedSender};

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

    pub fn editor_mut(&mut self, editor: usize) -> &mut EditorData {
        &mut self.editors[editor]
    }

    pub fn editors(&self) -> &[EditorData] {
        &self.editors
    }

    pub fn set_active_editor(&mut self, active_editor: usize) {
        self.active_editor = Some(active_editor);
    }

    pub fn remove_active_editor(&mut self) {
        self.active_editor = None;
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

    pub fn close_pane(&mut self, panel: usize) {
        self.panes.remove(panel);
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

        line.map(|line| Line {
            text: line.as_str().unwrap_or(""),
        })
    }
}

#[derive(Clone)]
pub struct EditorData {
    cursor: TextCursor,
    rope: Rope,
    path: PathBuf,
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

    fn insert(&mut self, value: &str, char_idx: usize) {
        self.rope.insert(char_idx, value);
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

        line.map(|line| Line {
            text: line.as_str().unwrap_or(""),
        })
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
}

pub type KeypressNotifier = UnboundedSender<Rc<KeyboardData>>;
pub type ClickNotifier = UnboundedSender<(Rc<MouseData>, usize)>;
pub type EditableText = UseState<Vec<Arc<Mutex<EditorData>>>>;

pub fn use_edit<'a>(
    cx: &'a ScopeState,
    editor_manager: &UseState<EditorManager>,
    pane_index: usize,
    editor_index: usize,
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
        let editor_manager = editor_manager.clone();
        let cursor_channels = cursor_channels.clone();
        async move {
            let cursor_receiver = cursor_channels.write().1.take();
            let mut cursor_receiver = cursor_receiver.unwrap();
            let cursor_ref = cursor_ref.clone();

            while let Some((new_cursor_col, new_cursor_row)) = cursor_receiver.recv().await {
                let editor = &editor_manager.current().panes[pane_index].editors[editor_index];

                let new_current_line = editor.rope.line(new_cursor_row);

                // Use the line lenght as new column if the clicked column surpases the length
                let new_cursor = if new_cursor_col >= new_current_line.chars().len() {
                    (new_current_line.chars().len(), new_cursor_row)
                } else {
                    (new_cursor_col, new_cursor_row)
                };

                // Only update if it's actually different
                if editor.cursor.as_tuple() != new_cursor {
                    editor_manager.with_mut(|editor_manager| {
                        let editor = &mut editor_manager.panes[pane_index].editors[editor_index];
                        editor.cursor.set_col(new_cursor.0);
                        editor.cursor.set_row(new_cursor.1);
                    });
                }

                // Remove the current calcutions so the layout engine doesn't try to calculate again
                cursor_ref.write().positions.lock().unwrap().take();
            }
        }
    });

    use_effect(cx, &pane_index, move |_| {
        let keypress_channel = keypress_channel.clone();
        let editor_manager = editor_manager.clone();
        async move {
            let rx = keypress_channel.write().1.take();
            let mut rx = rx.unwrap();

            while let Some(pressed_key) = rx.recv().await {
                if pressed_key.key == Key::Escape {
                    continue;
                }
                editor_manager.with_mut(|editor_manager| {
                    let editor = &mut editor_manager.panes[pane_index].editors[editor_index];
                    let event = editor.process_key(
                        &pressed_key.key,
                        &pressed_key.code,
                        &pressed_key.modifiers,
                    );
                    if event == TextEvent::TextChanged {
                        highlight_trigger.send(()).ok();
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
