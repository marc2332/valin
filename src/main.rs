use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use freya::prelude::events_data::KeyboardEvent;
use freya::prelude::*;

mod controlled_virtual_scroll_view;
mod use_editable;
mod use_syntax_highlighter;

use controlled_virtual_scroll_view::*;
use tokio::{fs::read_to_string, sync::mpsc::unbounded_channel};
pub use use_editable::{use_edit, EditableMode, EditableText};
use use_syntax_highlighter::*;

fn main() {
    launch_cfg(
        app,
        WindowConfig::<()>::builder()
            .with_width(900)
            .with_height(600)
            .with_title("Editor")
            .build(),
    );
}

fn app(cx: Scope) -> Element {
    use_init_focus(cx);
    render!(
        ThemeProvider {
            theme: DARK_THEME,
            Body {}
        }
    )
}

#[derive(Props, Clone)]
pub struct EditorProps<'a> {
    pub manager: &'a UseState<EditorManager>,
    pub panel_index: usize,
    pub editor: usize,
}

impl<'a> PartialEq for EditorProps<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.editor == other.editor
    }
}

#[allow(non_snake_case)]
fn Editor<'a>(cx: Scope<'a, EditorProps<'a>>) -> Element<'a> {
    let line_height_percentage = use_state(cx, || 0.0);
    let font_size_percentage = use_state(cx, || 15.0);
    let cursor = cx.props.manager.panes[cx.props.panel_index].editors[cx.props.editor]
        .lock()
        .unwrap()
        .cursor;
    let theme = use_theme(cx);
    let highlight_trigger = use_ref(cx, || {
        let (tx, rx) = unbounded_channel::<()>();
        (tx, Some(rx))
    });
    let (process_keyevent, process_clickevent, cursor_ref) = use_edit(
        cx,
        EditableMode::SingleLineMultipleEditors,
        cx.props.manager,
        cx.props.panel_index,
        cx.props.editor,
        highlight_trigger.read().0.clone(),
    );

    // Trigger initial highlighting
    use_effect(cx, (), move |_| {
        highlight_trigger.read().0.send(()).ok();
        async move {}
    });

    let syntax_blocks = use_syntax_highlighter(
        cx,
        cx.props.manager,
        cx.props.panel_index,
        cx.props.editor,
        highlight_trigger,
    );
    let scroll_y = use_state(cx, || 0);
    let destination_line = use_state(cx, String::new);
    let (focused, focus_id, focus) = use_raw_focus(cx);

    let font_size = font_size_percentage + 5.0;
    let line_height = (line_height_percentage / 25.0) + 1.2;
    let theme = theme.read();
    let manual_line_height = (font_size * line_height) as f32;

    let onkeydown = move |e: KeyboardEvent| {
        if focused {
            process_keyevent.send(e.data).ok();
        }
    };

    let onmousedown = move |_: MouseEvent| {
        *focus.unwrap().write() = focus_id;
    };

    let onscroll = move |(axis, scroll): (Axis, i32)| {
        if Axis::Y == axis {
            scroll_y.set(scroll)
        }
    };

    use_effect(cx, (), move |_| {
        *focus.unwrap().write() = focus_id;
        async move {}
    });

    render!(
        container {
            width: "100%",
            height: "80",
            padding: "20",
            direction: "horizontal",
            background: "rgb(20, 20, 20)",
            rect {
                height: "100%",
                width: "100%",
                direction: "horizontal",
                padding: "10",
                rect {
                    height: "40%",
                    display: "center",
                    width: "130",
                    Slider {
                        width: 100.0,
                        value: *font_size_percentage.get(),
                        onmoved: |p| {
                            font_size_percentage.set(p);
                        }
                    }
                    rect {
                        height: "auto",
                        width: "100%",
                        display: "center",
                        direction: "horizontal",
                        label {
                            "Font size"
                        }
                    }
                }
                rect {
                    height: "40%",
                    display: "center",
                    direction: "vertical",
                    width: "130",
                    Slider {
                        width: 100.0,
                        value: *line_height_percentage.get(),
                        onmoved: |p| {
                            line_height_percentage.set(p);
                        }
                    }
                    rect {
                        height: "auto",
                        width: "100%",
                        display: "center",
                        direction: "horizontal",
                        label {
                            "Line height"
                        }
                    }
                }
                Button {
                    onclick: move |_| {
                        if let Ok(v) = destination_line.get().parse::<i32>() {
                            scroll_y.set(-(manual_line_height * (v - 1) as f32) as i32);

                        }
                    },
                    label {
                        "Scroll to line:"
                    }
                }
                Input {
                    value: destination_line.get(),
                    onchange: move |v: String| {
                        if v.parse::<i32>().is_ok() || v.is_empty() {
                            destination_line.set(v);
                        }
                    }
                }
            }
        }
        rect {
            width: "100%",
            height: "calc(100% - 110)",
            onkeydown: onkeydown,
            onmousedown: onmousedown,
            cursor_reference: cursor_ref,
            direction: "horizontal",
            background: "{theme.body.background}",
            rect {
                width: "100%",
                height: "100%",
                ControlledVirtualScrollView {
                    scroll_x: 0,
                    scroll_y: *scroll_y.get(),
                    onscroll: onscroll,
                    width: "100%",
                    height: "100%",
                    show_scrollbar: true,
                    builder_values: (cursor, syntax_blocks),
                    length: syntax_blocks.len() as i32,
                    item_size: manual_line_height,
                    builder: Box::new(move |(k, line_index, args)| {
                        let (cursor, syntax_blocks) = args.unwrap();
                        let process_clickevent = process_clickevent.clone();
                        let line_index = line_index as usize;
                        let line = syntax_blocks.get().get(line_index).unwrap().clone();

                        let is_line_selected = cursor.1 == line_index;

                        // Only show the cursor in the active line
                        let character_index = if is_line_selected {
                            cursor.0.to_string()
                        } else {
                            "none".to_string()
                        };

                        // Only highlight the active line
                        let line_background = if is_line_selected {
                            "rgb(37, 37, 37)"
                        } else {
                            ""
                        };

                        let onmousedown = move |e: MouseEvent| {
                            process_clickevent.send((e.data, line_index)).ok();
                        };

                        rsx!(
                            rect {
                                key: "{k}",
                                width: "100%",
                                height: "{manual_line_height}",
                                direction: "horizontal",
                                background: "{line_background}",
                                radius: "7",
                                rect {
                                    width: "{font_size * 3.0}",
                                    height: "100%",
                                    display: "center",
                                    direction: "horizontal",
                                    label {
                                        font_size: "{font_size}",
                                        color: "rgb(200, 200, 200)",
                                        "{line_index + 1} "
                                    }
                                }
                                paragraph {
                                    width: "100%",
                                    cursor_index: "{character_index}",
                                    cursor_color: "white",
                                    max_lines: "1",
                                    cursor_mode: "editable",
                                    cursor_id: "{line_index}",
                                    onmousedown: onmousedown,
                                    height: "{manual_line_height}",
                                    line.iter().enumerate().map(|(i, (t, word))| {
                                        rsx!(
                                            text {
                                                key: "{i}",
                                                width: "100%",
                                                color: "{get_color_from_type(t)}",
                                                font_size: "{font_size}",
                                                "{word}"
                                            }
                                        )
                                    })
                                }
                            }
                        )
                    })
                }
            }
        }
        rect {
            width: "100%",
            height: "30",
            background: "rgb(20, 20, 20)",
            direction: "horizontal",
            padding: "10",
            label {
                color: "rgb(200, 200, 200)",
                "Ln {cursor.1 + 1}, Col {cursor.0 + 1}"
            }
        }
    )
}

#[derive(Clone)]
pub struct EditorData {
    cursor: (usize, usize),
    rope: Rope,
    path: PathBuf,
}

impl EditorData {
    pub fn new(path: PathBuf, rope: Rope, cursor: (usize, usize)) -> Self {
        Self { path, rope, cursor }
    }
}

#[derive(Clone, Default)]
pub struct Panel {
    active_editor: Option<usize>,
    editors: Vec<Arc<Mutex<EditorData>>>,
}

impl Panel {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone)]
pub struct EditorManager {
    focused_panel: usize,
    panes: Vec<Panel>,
}

impl Default for EditorManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorManager {
    pub fn new() -> Self {
        Self {
            focused_panel: 0,
            panes: vec![Panel::new()],
        }
    }

    pub fn push_editor(&mut self, editor: EditorData, panel: Option<usize>, focus: bool) {
        let panel = panel.unwrap_or(self.focused_panel);
        self.panes[panel].editors.push(Arc::new(Mutex::new(editor)));

        if focus {
            self.focused_panel = panel;
            self.panes[panel].active_editor = Some(self.panes[panel].editors.len() - 1);
        }
    }

    pub fn get_panes(&self) -> &[Panel] {
        &self.panes
    }

    pub fn get_editors(&self, panel: usize) -> &[Arc<Mutex<EditorData>>] {
        &self.panes[panel].editors
    }

    pub fn get_active_editor(&self, panel: usize) -> Option<usize> {
        self.panes[panel].active_editor
    }

    pub fn set_active_editor(&mut self, panel: usize, active_editor: usize) {
        self.panes[panel].active_editor = Some(active_editor);
    }

    pub fn set_focused_pane(&mut self, panel: usize) {
        self.focused_panel = panel;
    }

    pub fn get_focused_pane(&self) -> usize {
        self.focused_panel
    }
}

#[allow(non_snake_case)]
fn Body(cx: Scope) -> Element {
    let theme = use_theme(cx);
    let theme = &theme.read();
    let editor_manager = use_state::<EditorManager>(cx, EditorManager::new);

    let open_file = move |_: MouseEvent| {
        let editor_manager = editor_manager.clone();
        cx.spawn(async move {
            let task = rfd::AsyncFileDialog::new().pick_file();
            let file = task.await;

            if let Some(file) = file {
                let path = file.path();
                let content = read_to_string(&path).await.unwrap();
                editor_manager.with_mut(|editor_manager| {
                    editor_manager.push_editor(
                        EditorData::new(path.to_path_buf(), Rope::from(content), (0, 0)),
                        None,
                        true,
                    );
                });
            }
        });
    };

    let create_panel = |_| {
        editor_manager.with_mut(|editor_manager| {
            editor_manager.panes.push(Panel::new());
        });
    };

    let pane_size = 100.0 / editor_manager.get().get_panes().len() as f32;

    render!(
        rect {
            background: "rgb(20, 20, 20)",
            direction: "horizontal",
            width: "100%",
            height: "100%",
            rect {
                direction: "vertical",
                width: "60",
                height: "100%",
                Button {
                    onclick: open_file,
                    label {
                        "Open"
                    }
                }
                Button {
                    onclick: create_panel,
                    label {
                        "Split"
                    }
                }
            }
            rect {
                background: "rgb(100, 100, 100)",
                height: "100%",
                width: "2",
            }
            rect {
                height: "100%",
                width: "calc(100% - 62)",
                direction: "horizontal",
                editor_manager.get().get_panes().iter().enumerate().map(|(panel_index, panel)| {
                    let is_focused = editor_manager.get().get_focused_pane() == panel_index;
                    let active_editor = panel.active_editor;
                    let bg = if is_focused {
                        "rgb(247, 127, 0)"
                    } else {
                        "transparent"
                    };
                    rsx!(
                        rect {
                            direction: "vertical",
                            height: "100%",
                            width: "{pane_size}%",
                            rect {
                                direction: "horizontal",
                                height: "50",
                                width: "100%",
                                padding: "5",
                                editor_manager.get().get_editors(panel_index).iter().enumerate().map(|(i, editor)| {
                                    let path = &editor.lock().unwrap().path;
                                    let is_selected = active_editor == Some(i);
                                    let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();
                                    rsx!(
                                        FileTab {
                                            key: "{i}",
                                            onclick: move |_| {
                                                editor_manager.with_mut(|editor_manager| {
                                                    editor_manager.set_focused_pane(panel_index);
                                                    editor_manager.set_active_editor(panel_index, i);
                                                });
                                            },
                                            value: "{file_name}",
                                            is_selected: is_selected
                                        }
                                    )
                                })
                            }
                            rect {
                                height: "calc(100%-50)",
                                width: "100%",
                                background: "{bg}",
                                padding: "3",
                                onclick: move |_| {
                                    editor_manager.with_mut(|editor_manager| {
                                        editor_manager.set_focused_pane(panel_index);
                                    });
                                },
                                if let Some(active_editor) = active_editor {
                                    rsx!(
                                        Editor {
                                            key: "{active_editor}",
                                            manager: editor_manager,
                                            panel_index: panel_index,
                                            editor: active_editor
                                        }
                                    )
                                } else {
                                    rsx!(
                                        rect {
                                            display: "center",
                                            width: "100%",
                                            height: "100%",
                                            direction: "both",
                                            background: "{theme.body.background}",
                                            label {
                                                "Open a file!"
                                            }
                                        }
                                    )
                                }
                            }

                        }
                    )
                })
            }

        }
    )
}

#[allow(non_snake_case)]
#[inline_props]
fn FileTab<'a>(
    cx: Scope<'a>,
    value: &'a str,
    onclick: EventHandler<(), 'a>,
    is_selected: bool,
) -> Element {
    let theme = use_get_theme(cx);
    let button_theme = &theme.button;

    let background = use_state(cx, || <&str>::clone(&button_theme.background));
    let set_background = background.setter();

    use_effect(cx, &button_theme.clone(), move |button_theme| async move {
        set_background(button_theme.background);
    });

    let selected_background = if *is_selected {
        button_theme.hover_background
    } else {
        background.get()
    };

    render!(
        rect {
            padding: "4",
            width: "150",
            height: "100%",
            rect {
                color: "{button_theme.font_theme.color}",
                background: "{selected_background}",
                shadow: "0 5 15 10 black",
                radius: "5",
                onclick: move |_| onclick.call(()),
                onmouseover: move |_| {
                    background.set(theme.button.hover_background);
                },
                onmouseleave: move |_| {
                    background.set(theme.button.background);
                },
                padding: "15",
                width: "100%",
                height: "100%",
                display: "center",
                direction: "both",
                label {
                    "{value}"
                }
            }
        }
    )
}
