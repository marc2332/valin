use freya::prelude::events_data::KeyboardEvent;
use freya::prelude::*;

mod controlled_virtual_scroll_view;
mod use_editable;
mod use_syntax_highlighter;

use controlled_virtual_scroll_view::*;
use tokio::{fs::read_to_string, sync::mpsc::unbounded_channel};
pub use use_editable::{use_edit, EditableText};
use use_editable::{EditorData, EditorManager, Panel};
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

#[derive(Props)]
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
    let cursor = cx
        .props
        .manager
        .panel(cx.props.panel_index)
        .editor(cx.props.editor)
        .cursor();
    let theme = use_theme(cx);
    let highlight_trigger = use_ref(cx, || {
        let (tx, rx) = unbounded_channel::<()>();
        (tx, Some(rx))
    });
    let (process_keyevent, process_clickevent, cursor_ref) = use_edit(
        cx,
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
    let scroll_x = use_state(cx, || 0);
    let scroll_y = use_state(cx, || 0);
    let destination_line = use_state(cx, String::new);

    let theme = theme.read();
    let font_size = cx.props.manager.font_size();
    let manual_line_height = cx.props.manager.font_size() * cx.props.manager.line_height();
    let is_panel_focused = cx.props.manager.focused_panel() == cx.props.panel_index;
    let is_editor_focused =
        cx.props.manager.panel(cx.props.panel_index).active_editor() == Some(cx.props.editor);

    let onkeydown = move |e: KeyboardEvent| {
        if is_editor_focused && is_panel_focused {
            process_keyevent.send(e.data).ok();
        }
    };

    let onmousedown = move |_: MouseEvent| {
        if !is_editor_focused {
            cx.props.manager.with_mut(|manager| {
                manager.set_focused_panel(cx.props.panel_index);
                manager
                    .panel_mut(cx.props.panel_index)
                    .set_active_editor(cx.props.editor);
            });
        }
    };

    let onscroll = move |(axis, scroll): (Axis, i32)| match axis {
        Axis::Y => scroll_y.set(scroll),
        Axis::X => scroll_x.set(scroll),
    };

    use_effect(cx, (), move |_| {
        cx.props.manager.with_mut(|manager| {
            manager.set_focused_panel(cx.props.panel_index);
            manager
                .panel_mut(cx.props.panel_index)
                .set_active_editor(cx.props.editor);
        });
        async move {}
    });

    render!(
        container {
            width: "100%",
            height: "60",
            padding: "10",
            direction: "horizontal",
            background: "rgb(20, 20, 20)",
            rect {
                height: "100%",
                width: "100%",
                direction: "horizontal",
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
            height: "calc(100% - 90)",
            onkeydown: onkeydown,
            onmousedown: onmousedown,
            cursor_reference: cursor_ref,
            direction: "horizontal",
            background: "{theme.body.background}",
            rect {
                width: "100%",
                height: "100%",
                ControlledVirtualScrollView {
                    scroll_x: *scroll_x.get(),
                    scroll_y: *scroll_y.get(),
                    onscroll: onscroll,
                    width: "100%",
                    height: "100%",
                    show_scrollbar: true,
                    builder_values: (cursor.clone(), syntax_blocks),
                    length: syntax_blocks.len() as i32,
                    item_size: manual_line_height,
                    builder: Box::new(move |(k, line_index, args)| {
                        let (cursor, syntax_blocks) = args.as_ref().unwrap();
                        let process_clickevent = process_clickevent.clone();
                        let line_index = line_index as usize;
                        let line = syntax_blocks.get().get(line_index).unwrap();

                        let is_line_selected = cursor.row() == line_index;

                        // Only show the cursor in the active line
                        let character_index = if is_line_selected {
                            cursor.col().to_string()
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
                                    direction: "horizontal",
                                    label {
                                        width: "100%",
                                        align: "center",
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
                                    direction: "horizontal",
                                    line.iter().enumerate().map(|(i, (t, word))| {
                                        rsx!(
                                            text {
                                                font_family: "Jetbrains Mono",
                                                key: "{i}",
                                                width: "auto",
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
            padding: "5",
            label {
                color: "rgb(200, 200, 200)",
                "Ln {cursor.row() + 1}, Col {cursor.col() + 1}"
            }
        }
    )
}

#[allow(non_snake_case)]
fn Body(cx: Scope) -> Element {
    let theme = use_theme(cx);
    let theme = &theme.read();
    let editor_manager = use_state::<EditorManager>(cx, EditorManager::new);

    let (commander_anim, _, commander_height, _) = use_animation_managed(cx, 0.0);

    let onkeydown = move |e: KeyboardEvent| {
        if e.code == Code::Escape {
            if commander_height == 0.0 {
                commander_anim(AnimationMode::new_sine_in_out(0.0..=50.0, 100))
            } else {
                commander_anim(AnimationMode::new_sine_in_out(50.0..=0.0, 100))
            }
        }
    };

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
                        editor_manager.focused_panel(),
                        true,
                    );
                });
            }
        });
    };

    let create_panel = |_| {
        editor_manager.with_mut(|editor_manager| {
            editor_manager.push_panel(Panel::new());
            editor_manager.set_focused_panel(editor_manager.panels().len() - 1);
        });
    };

    let pane_size = 100.0 / editor_manager.get().panels().len() as f32;

    render!(
        rect {
            onkeydown: onkeydown,
            color: "white",
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
                direction: "vertical",
                width: "calc(100% - 62)",
                height: "100%",
                rect {
                    height: "calc(100% - {commander_height})",
                    width: "calc(100%)",
                    direction: "horizontal",
                    editor_manager.get().panels().iter().enumerate().map(|(panel_index, panel)| {
                        let is_focused = editor_manager.get().get_focused_pane() == panel_index;
                        let active_editor = panel.active_editor();
                        let bg = if is_focused {
                            "rgb(247, 127, 0)"
                        } else {
                            "transparent"
                        };
                        let close_panel = move |_: MouseEvent| {
                            editor_manager.with_mut(|editor_manager| {
                                editor_manager.close_pane(panel_index);
                            });
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
                                    padding: "2.5",
                                    editor_manager.get().panel(panel_index).editors().iter().enumerate().map(|(i, editor)| {
                                        let is_selected = active_editor == Some(i);
                                        let file_name = editor.path().file_name().unwrap().to_str().unwrap().to_owned();
                                        rsx!(
                                            FileTab {
                                                key: "{i}",
                                                onclick: move |_| {
                                                    editor_manager.with_mut(|editor_manager| {
                                                        editor_manager.set_focused_panel(panel_index);
                                                        editor_manager.panel_mut(panel_index).set_active_editor(i);
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
                                    padding: "1.5",
                                    onclick: move |_| {
                                        editor_manager.with_mut(|editor_manager| {
                                            editor_manager.set_focused_panel(panel_index);
                                        });
                                    },
                                    if let Some(active_editor) = active_editor {
                                        rsx!(
                                            Editor {
                                                key: "{active_editor}",
                                                manager: editor_manager,
                                                panel_index: panel_index,
                                                editor: active_editor,
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
                                                Button {
                                                    onclick: close_panel,
                                                    label {
                                                        "Close panel"
                                                    }
                                                }
                                            }
                                        )
                                    }
                                }

                            }
                        )
                    })
                }
                Commander {
                    height: commander_height
                }
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
            padding: "2",
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
                padding: "7",
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

#[allow(non_snake_case)]
#[inline_props]
fn Commander(cx: Scope, height: f64) -> Element {
    render!(
        container {
            width: "100%",
            height: "{height}",
            display: "center",
            direction: "vertical",
            padding: "0 25",
            label {
                "Command"
            }
        }
    )
}
