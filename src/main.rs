use freya::prelude::*;
use freya::prelude::events::KeyboardEvent;

mod use_syntax_highlighter;

use use_syntax_highlighter::*;

fn main() {
    launch_cfg(vec![(
        app,
        WindowConfig::<()>::builder()
            .with_width(900)
            .with_height(500)
            .with_title("Editor")
            .build(),
    )]);
}

fn app(cx: Scope) -> Element {
    render!(
        ThemeProvider {
            theme: DARK_THEME,
            Body {}
        }
    )
}

const DUMMY_CODE: &str = "const test = false;\n\nfunction test(val = 123){\n   console.log(val);\n   let data = `multi\n   line`;\n}\n\n";

#[allow(non_snake_case)]
fn Body(cx: Scope) -> Element {
    // Hooks
    let line_height_percentage = use_state(&cx, || 0.0);
    let font_size_percentage = use_state(&cx, || 15.0);
    let code = cx.use_hook(|| DUMMY_CODE.repeat(400));
    let is_italic = use_state(&cx, || false);
    let is_bold = use_state(&cx, || false);
    let theme = use_theme(&cx);
    let (content, cursor, process_keyevent, process_clickevent, cursor_ref) =
        use_editable(&cx, || code, EditableMode::SingleLineMultipleEditors);
    let syntax_blocks = use_syntax_highlighter(&cx, content);

    // Simple calculations
    let font_size = font_size_percentage + 5.0;
    let line_height = (line_height_percentage / 25.0) + 1.2;
    let font_style = {
        if *is_bold.get() && *is_italic.get() {
            "bold-italic"
        } else if *is_italic.get() {
            "italic"
        } else if *is_bold.get() {
            "bold"
        } else {
            "normal"
        }
    };
    let theme = theme.read();
    let manual_line_height = (font_size * line_height) as f32;

    let onkeydown = move |e: KeyboardEvent| {
        process_keyevent.send(e.data).ok();
    };

    render!(
        rect {
            width: "100%",
            height: "60",
            padding: "20",
            direction: "horizontal",
            background: "rgb(20, 20, 20)",
            rect {
                height: "100%",
                width: "100%",
                direction: "horizontal",
                padding: "10",
                label {
                    font_size: "30",
                    "Editor"
                }
                rect {
                    width: "20",
                }
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
                rect {
                    height: "40%",
                    display: "center",
                    direction: "vertical",
                    width: "60",
                    Switch {
                        enabled: *is_bold.get(),
                        ontoggled: |_| {
                            is_bold.set(!is_bold.get());
                        }
                    }
                    rect {
                        height: "auto",
                        width: "100%",
                        display: "center",
                        direction: "horizontal",
                        label {
                            "Bold"
                        }
                    }
                }
                rect {
                    height: "40%",
                    display: "center",
                    direction: "vertical",
                    width: "60",
                    Switch {
                        enabled: *is_italic.get(),
                        ontoggled: |_| {
                            is_italic.set(!is_italic.get());
                        }
                    }
                    rect {
                        height: "auto",
                        width: "100%",
                        display: "center",
                        direction: "horizontal",
                        label {
                            "Italic"
                        }
                    }
                }
            }
        }
        rect {
            width: "100%",
            height: "calc(100% - 90)",
            padding: "20",
            onkeydown: onkeydown,
            cursor_reference: cursor_ref,
            direction: "horizontal",
            background: "{theme.body.background}",
            rect {
                width: "100%",
                height: "100%",
                padding: "30",
                VirtualScrollView {
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
                                                font_style: "{font_style}",
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
