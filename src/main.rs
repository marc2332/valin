use freya::prelude::*;
use tree_sitter_highlight::*;

fn main() {
    launch_cfg(vec![(
        app,
        WindowConfig::<()>::builder()
            .with_width(900)
            .with_height(500)
            .with_decorations(true)
            .with_transparency(false)
            .with_title("Editor")
            .build(),
    )]);
}

#[derive(Clone)]
pub enum SyntaxType {
    Number,
    String,
    Keyword,
    Operator,
    Variable,
    Unknown,
}

impl From<&str> for SyntaxType {
    fn from(s: &str) -> Self {
        match s {
            "keyword" => SyntaxType::Keyword,
            "variable" => SyntaxType::Variable,
            "operator" => SyntaxType::Operator,
            "string" => SyntaxType::String,
            "number" => SyntaxType::Number,
            _ => SyntaxType::Unknown,
        }
    }
}

type SyntaxBlocks = Vec<Vec<(SyntaxType, String)>>;

fn use_syntax_highlighter<'a>(
    cx: &'a ScopeState,
    content: &EditableText,
) -> &'a UseState<SyntaxBlocks> {
    let syntax_blocks = use_state::<SyntaxBlocks>(cx, Vec::new);
    let highlighter = cx.use_hook(Highlighter::new);

    use_effect(cx, &content.len(), move |_| {
        let highlight_names = &mut [
            "attribute",
            "constant",
            "function.builtin",
            "function",
            "keyword",
            "operator",
            "property",
            "punctuation",
            "punctuation.bracket",
            "punctuation.delimiter",
            "string",
            "string.special",
            "tag",
            "type",
            "type.builtin",
            "variable",
            "variable.builtin",
            "variable.parameter",
            "number",
        ];

        let mut javascript_config = HighlightConfiguration::new(
            tree_sitter_javascript::language(),
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            tree_sitter_javascript::INJECTION_QUERY,
            tree_sitter_javascript::LOCALS_QUERY,
        )
        .unwrap();

        javascript_config.configure(highlight_names);

        let data = content.to_string();
        let highlights = highlighter
            .highlight(&javascript_config, data.as_bytes(), None, |_| None)
            .unwrap();

        syntax_blocks.with_mut(|syntax_blocks| {
            syntax_blocks.clear();
            let mut prepared_block: (SyntaxType, Vec<(usize, String)>) =
                (SyntaxType::Unknown, vec![]);

            for event in highlights {
                match event.unwrap() {
                    HighlightEvent::Source { start, end } => {
                        // Prepare the whole block even if it's splitted across multiple lines.
                        let data = content.get().lines(start..end);
                        let starting_line = content.get().line_of_offset(start);

                        for (i, d) in data.enumerate() {
                            prepared_block.1.push((starting_line + i, d.to_string()));
                        }
                    }
                    HighlightEvent::HighlightStart(s) => {
                        // Specify the type of the block
                        prepared_block.0 = SyntaxType::from(highlight_names[s.0]);
                    }
                    HighlightEvent::HighlightEnd => {
                        // Push all the block chunks to their specified line
                        for (i, d) in prepared_block.1 {
                            if syntax_blocks.get(i).is_none() {
                                syntax_blocks.push(vec![]);
                            }
                            let line = syntax_blocks.last_mut().unwrap();
                            line.push((prepared_block.0.clone(), d));
                        }
                        // Clear the prepared block
                        prepared_block = (SyntaxType::Unknown, vec![]);
                    }
                }
            }

            // Mark all the remaining text as not readable
            if !prepared_block.1.is_empty() {
                for (i, d) in prepared_block.1 {
                    if syntax_blocks.get(i).is_none() {
                        syntax_blocks.push(vec![]);
                    }
                    let line = syntax_blocks.last_mut().unwrap();
                    line.push((SyntaxType::Unknown, d));
                }
            }
        });

        async move {}
    });

    syntax_blocks
}

const DUMMY_CODE: &str = "const test = false;\n\nfunction test(val = 123){\n   console.log(val);\n   let data = `multi\n   line`;\n}\n\n";

#[allow(non_snake_case)]
fn Body(cx: Scope) -> Element {
    let dummy_code = cx.use_hook(|| DUMMY_CODE.repeat(400));
    let theme = use_theme(&cx);
    let font_size_percentage = use_state(&cx, || 15.0);
    let line_height_percentage = use_state(&cx, || 0.0);
    let is_bold = use_state(&cx, || false);
    let is_italic = use_state(&cx, || false);

    let (content, cursor, process_keyevent, process_clickevent, cursor_ref) =
        use_editable(&cx, || dummy_code, EditableMode::SingleLineMultipleEditors);
    let syntax_blocks = use_syntax_highlighter(&cx, content);

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
            onkeydown: process_keyevent,
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
                            process_clickevent.send((e, line_index)).ok();
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

fn app(cx: Scope) -> Element {
    use_init_theme(&cx, DARK_THEME);
    render!(Body {})
}

fn get_color_from_type<'a>(t: &SyntaxType) -> &'a str {
    match t {
        SyntaxType::Keyword => "rgb(248, 73, 52)",
        SyntaxType::Variable => "rgb(189, 174, 147)",
        SyntaxType::Operator => "rgb(189, 174, 147)",
        SyntaxType::String => "rgb(184, 187, 38)",
        SyntaxType::Number => "rgb(211, 134, 155)",
        SyntaxType::Unknown => "rgb(189, 174, 147)",
    }
}
