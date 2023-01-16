use std::path::PathBuf;

use freya::prelude::events_data::KeyboardEvent;
use freya::prelude::*;

mod use_editable;
mod use_syntax_highlighter;
use tokio::fs::read_to_string;
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
    pub editables: &'a EditableText,
    pub editable_index: usize,
}

impl<'a> PartialEq for EditorProps<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.editable_index == other.editable_index
    }
}

#[allow(non_snake_case)]
fn Editor<'a>(cx: Scope<'a, EditorProps<'a>>) -> Element<'a> {
    let line_height_percentage = use_state(cx, || 0.0);
    let font_size_percentage = use_state(cx, || 15.0);
    let (_, content, cursor) = cx
        .props
        .editables
        .get()
        .get(cx.props.editable_index)
        .unwrap();
    let theme = use_theme(cx);
    let (process_keyevent, process_clickevent, cursor_ref) = use_edit(
        cx,
        EditableMode::SingleLineMultipleEditors,
        cx.props.editables,
        cx.props.editable_index,
    );

    // I could pass a vector of type UseState but what about using Atoms¿?

    let syntax_blocks = use_syntax_highlighter(cx, content);
    let scroll_y = use_state(cx, || 0);
    let destination_line = use_state(cx, String::new);
    let (focused, focus_id, focus) = use_raw_focus(cx);

    // Simple calculations
    let font_size = font_size_percentage + 5.0;
    let line_height = (line_height_percentage / 25.0) + 1.2;
    let theme = theme.read();
    let manual_line_height = (font_size * line_height) as f32;

    let onkeydown = move |e: KeyboardEvent| {
        if focused {
            process_keyevent.send(e.data).ok();
        }
    };

    let onclick = move |_: MouseEvent| {
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
            onclick: onclick,
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

#[allow(non_snake_case)]
fn Body(cx: Scope) -> Element {
    let tabs = use_state::<Vec<(PathBuf, Rope, (usize, usize))>>(cx, Vec::new);
    let selected_tab = use_state::<Option<usize>>(cx, || None);

    let open_file = move |_: MouseEvent| {
        let tabs = tabs.clone();
        let selected_tab = selected_tab.clone();
        cx.spawn(async move {
            let task = rfd::AsyncFileDialog::new().pick_file();
            let file = task.await;

            if let Some(file) = file {
                let path = file.path();
                let content = read_to_string(&path).await.unwrap();
                tabs.with_mut(|tabs| {
                    tabs.push((path.to_path_buf(), Rope::from(content), (0, 0)));
                    selected_tab.set(Some(tabs.len() - 1));
                });
            }
        });
    };

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
            }
            rect {
                background: "rgb(100, 100, 100)",
                height: "100%",
                width: "2",
            }
            rect {
                direction: "vertical",
                height: "100%",
                width: "calc(100%-62)",
                rect {
                    direction: "horizontal",
                    height: "60",
                    width: "100%",
                    tabs.get().iter().enumerate().map(|(i, (path, _, _))| {
                        rsx!(
                            Button {
                                key: "{i}",
                                onclick: move |_| {
                                    selected_tab.set(Some(i));
                                },
                                label {
                                    "{path.file_name().unwrap().to_str().unwrap()}"
                                }
                            }
                        )
                    })
                }
                rect {
                    height: "calc(100%-60)",
                    width: "100%",
                    if let Some(selected_tab) = selected_tab.get() {
                        rsx!(
                            Editor {
                                key: "{selected_tab}",
                                editables: tabs,
                                editable_index: *selected_tab
                            }
                        )
                    } else {
                        rsx!(
                            rect {
                                display: "center",
                                width: "100%",
                                height: "100%",
                                direction: "both",
                                label {
                                    "Open a file!"
                                }
                            }
                        )
                    }
                }

            }
        }
    )
}
