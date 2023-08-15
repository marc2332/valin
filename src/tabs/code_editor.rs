use std::path::PathBuf;
use std::time::Duration;

use crate::controlled_virtual_scroll_view::*;
use crate::lsp::LspConfig;
use crate::manager::use_manager;
use crate::manager::EditorManager;
use crate::parser::SyntaxBlocks;
use crate::use_debouncer::use_debouncer;
use crate::use_debouncer::UseDebouncer;
use crate::use_editable;
use crate::use_editable::*;
use crate::use_metrics::*;
use crate::utils::create_paragraph;
use async_lsp::LanguageServer;
use freya::prelude::events::KeyboardEvent;
use freya::prelude::*;
use lsp_types::Hover;
use lsp_types::MarkedString;

use lsp_types::{
    DidOpenTextDocumentParams, HoverContents, HoverParams, Position, TextDocumentIdentifier,
    TextDocumentItem, TextDocumentPositionParams, Url, WorkDoneProgressParams,
};
use tokio_stream::StreamExt;

#[derive(Props, PartialEq)]
pub struct EditorProps {
    pub panel_index: usize,
    pub editor: usize,
    pub language_id: String,
    pub root_path: PathBuf,
}

pub enum LspAction {
    Hover(HoverParams),
    Clear,
}

#[allow(non_snake_case)]
pub fn CodeEditorTab(cx: Scope<EditorProps>) -> Element {
    let lsp_config = LspConfig::new(cx.props.root_path.clone(), &cx.props.language_id);
    let manager = use_manager(cx);
    let debouncer = use_debouncer(cx, Duration::from_millis(300));

    cx.use_hook(|| {
        to_owned![lsp_config, manager];

        // Focus editor
        {
            let mut manager = manager.write();
            manager.set_focused_panel(cx.props.panel_index);
            manager
                .panel_mut(cx.props.panel_index)
                .set_active_tab(cx.props.editor);
        }

        let (file_uri, file_text) = {
            let manager = manager.current();

            let editor = manager
                .panel(cx.props.panel_index)
                .tab(cx.props.editor)
                .as_text_editor()
                .unwrap();

            let path = editor.path();
            (
                Url::from_file_path(path).unwrap(),
                editor.rope().to_string(),
            )
        };

        // Notify language server the file has been opened
        cx.spawn(async move {
            let mut lsp = EditorManager::get_or_insert_lsp(manager, &lsp_config).await;

            lsp.server_socket
                .did_open(DidOpenTextDocumentParams {
                    text_document: TextDocumentItem {
                        uri: file_uri,
                        language_id: "rust".into(),
                        version: 0,
                        text: file_text,
                    },
                })
                .unwrap();
        });
    });

    let hover = use_ref(cx, || None);
    let (metrics, coroutine_coroutine) =
        use_metrics(cx, &manager, cx.props.panel_index, cx.props.editor);
    let editable = use_edit(
        cx,
        &manager,
        cx.props.panel_index,
        cx.props.editor,
        coroutine_coroutine,
    );
    let cursor_coords = use_ref::<CursorPoint>(cx, CursorPoint::default);
    let offset_x = use_state(cx, || 0);
    let offset_y = use_state(cx, || 0);

    let lsp_coroutine = use_coroutine(cx, |mut rx: UnboundedReceiver<LspAction>| {
        to_owned![lsp_config, hover, manager];
        async move {
            while let Some(action) = rx.next().await {
                match action {
                    LspAction::Hover(params) => {
                        let lsp = manager.current().lsp(&lsp_config).cloned();

                        if let Some(mut lsp) = lsp {
                            let is_indexed = *lsp.indexed.lock().unwrap();
                            if is_indexed {
                                let line = params.text_document_position_params.position.line;
                                let response = lsp.server_socket.hover(params).await;

                                if let Ok(Some(res)) = response {
                                    *hover.write() = Some((line, res));
                                } else {
                                    *hover.write() = None;
                                }
                            } else {
                                println!("LSP: Still indexing...");
                            }
                        } else {
                            println!("LSP: Not running.");
                        }
                    }
                    LspAction::Clear => {
                        *hover.write() = None;
                    }
                }
            }
        }
    });

    let manager_ref = manager.current();
    let cursor_attr = editable.cursor_attr(cx);
    let font_size = manager_ref.font_size();
    let manual_line_height = manager_ref.font_size() * manager_ref.line_height();
    let is_panel_focused = manager_ref.focused_panel() == cx.props.panel_index;
    let panel = manager_ref.panel(cx.props.panel_index);
    let is_editor_focused = manager_ref.is_focused() && panel.active_tab() == Some(cx.props.editor);
    let editor = panel.tab(cx.props.editor).as_text_editor().unwrap();
    let path = editor.path();
    let cursor = editor.cursor();
    let file_uri = Url::from_file_path(path).unwrap();

    let onmousedown = {
        to_owned![manager];
        move |_: MouseEvent| {
            if !is_editor_focused {
                let mut manager = manager.global_write();
                manager.set_focused_panel(cx.props.panel_index);
                manager
                    .panel_mut(cx.props.panel_index)
                    .set_active_tab(cx.props.editor);
            }
        }
    };

    let onscroll = move |(axis, scroll): (Axis, i32)| match axis {
        Axis::Y => offset_y.set(scroll),
        Axis::X => offset_x.set(scroll),
    };

    let onclick = {
        to_owned![editable];
        move |_: MouseEvent| {
            if is_panel_focused {
                editable.process_event(&EditableEvent::Click);
            }
        }
    };

    let onkeydown = {
        to_owned![editable];
        move |e: KeyboardEvent| {
            if is_panel_focused && is_editor_focused {
                editable.process_event(&EditableEvent::KeyDown(e.data));
            }
        }
    };

    render!(
        rect {
            width: "100%",
            height: "100%",
            onkeydown: onkeydown,
            onglobalclick: onclick,
            onmousedown: onmousedown,
            cursor_reference: cursor_attr,
            direction: "horizontal",
            background: "rgb(40, 40, 40)",
            ControlledVirtualScrollView {
                offset_x: *offset_x.get(),
                offset_y: *offset_y.get(),
                onscroll: onscroll,
                builder_values: (cursor, metrics.clone(), editable, lsp_coroutine.clone(), file_uri, editor.rope(), hover.clone(), cursor_coords.clone(), debouncer.clone()),
                length: metrics.0.len(),
                item_size: manual_line_height,
                builder: Box::new(move |(k, line_index, _cx, options)| {
                    rsx!(
                        EditorLine {
                            key: "{k}",
                            line_index: line_index,
                            options: options,
                            font_size: font_size,
                            line_height: manual_line_height
                        }
                    )
                })
            }
        }
    )
}

type BuilderProps<'a> = (
    TextCursor,
    UseState<(SyntaxBlocks, f32)>,
    use_editable::UseEdit,
    Coroutine<LspAction>,
    Url,
    &'a Rope,
    UseRef<Option<(u32, Hover)>>,
    UseRef<CursorPoint>,
    UseDebouncer,
);

#[allow(non_snake_case)]
#[inline_props]
fn EditorLine<'a>(
    cx: Scope,
    options: &'a BuilderProps<'a>,
    line_index: usize,
    font_size: f32,
    line_height: f32,
) -> Element {
    let (cursor, metrics, editable, lsp_coroutine, file_uri, rope, hover, cursor_coords, debouncer) =
        options;
    let (syntax_blocks, width) = metrics.get();
    let line = syntax_blocks.get(*line_index).unwrap();
    let line_str = rope.line(*line_index).to_string();
    let highlights_attr = editable.highlights_attr(cx, *line_index);

    let is_line_selected = cursor.row() == *line_index;

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

    let onmousedown = {
        to_owned![editable];
        move |e: MouseEvent| {
            editable.process_event(&EditableEvent::MouseDown(e.data, *line_index));
        }
    };

    let onmouseleave = |_| {
        lsp_coroutine.send(LspAction::Clear);
    };

    let onmouseover = {
        to_owned![editable, file_uri, lsp_coroutine, cursor_coords, hover];
        move |e: MouseEvent| {
            let coords = e.get_element_coordinates();
            let data = e.data;

            editable.process_event(&EditableEvent::MouseOver(data, *line_index));

            // Optimization: Re run the component only when the hover box is shown
            // otherwise just update the coordinates silently
            if hover.read().is_some() {
                *cursor_coords.write() = coords;
            } else {
                *cursor_coords.write_silent() = coords;
            }

            let paragraph = create_paragraph(&line_str, *font_size);

            if (coords.x as f32) < paragraph.max_intrinsic_width() {
                to_owned![cursor_coords, file_uri, lsp_coroutine, line_index];
                debouncer.action(move || {
                    let coords = cursor_coords.read();
                    let glyph = paragraph
                        .get_glyph_position_at_coordinate((coords.x as i32, coords.y as i32));

                    lsp_coroutine.send(LspAction::Hover(HoverParams {
                        text_document_position_params: TextDocumentPositionParams {
                            text_document: TextDocumentIdentifier {
                                uri: file_uri.clone(),
                            },
                            position: Position::new(line_index as u32, glyph.position as u32),
                        },
                        work_done_progress_params: WorkDoneProgressParams::default(),
                    }));
                });
            } else {
                lsp_coroutine.send(LspAction::Clear);
            }
        }
    };

    let gutter_width = font_size * 3.0;

    render!(
        if let Some((line, hover)) = hover.read().as_ref() {
            if *line == *line_index as u32 {
                if let Some(content) = hover.hover_to_text() {
                    let cursor_coords = cursor_coords.read();
                    let offset_x = cursor_coords.x  as f32 + gutter_width;
                        Some(rsx!(
                            rect {
                                width: "0",
                                height: "0",
                                offset_y: "{line_height}",
                                offset_x: "{offset_x}",
                                HoverBox {
                                    content: content
                                }
                            }
                        ))
                }  else {
                    None
                }
            } else {
                None
            }
        }
        rect {
            height: "{line_height}",
            direction: "horizontal",
            background: "{line_background}",
            rect {
                width: "{gutter_width}",
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
                min_width: "calc(100% - {gutter_width})",
                width: "{width}",
                cursor_index: "{character_index}",
                cursor_color: "white",
                max_lines: "1",
                cursor_mode: "editable",
                cursor_id: "{line_index}",
                onmousedown: onmousedown,
                onmouseover: onmouseover,
                onmouseleave: onmouseleave,
                highlights: highlights_attr,
                highlight_color: "rgb(65, 65, 65)",
                direction: "horizontal",
                font_size: "{font_size}",
                font_family: "Jetbrains Mono",
                line.iter().enumerate().map(|(i, (syntax_type, word))| {
                    rsx!(
                        text {
                            key: "{i}",
                            color: "{syntax_type.color()}",
                            "{word}"
                        }
                    )
                })
            }
        }
    )
}

trait HoverToText {
    fn hover_to_text(&self) -> Option<String>;
}

impl HoverToText for Hover {
    fn hover_to_text(&self) -> Option<String> {
        let text = match &self.contents {
            HoverContents::Markup(contents) => contents.value.to_owned(),
            HoverContents::Array(contents) => contents
                .iter()
                .map(|v| match v {
                    MarkedString::String(v) => v.to_owned(),
                    MarkedString::LanguageString(text) => text.value.to_owned(),
                })
                .collect::<Vec<String>>()
                .join("\n"),
            HoverContents::Scalar(v) => match v {
                MarkedString::String(v) => v.to_owned(),
                MarkedString::LanguageString(text) => text.value.to_owned(),
            },
        };

        if text == "()" {
            None
        } else {
            Some(text)
        }
    }
}

#[allow(non_snake_case)]
#[inline_props]
fn HoverBox(cx: Scope, content: String) -> Element {
    let height = match content.trim().lines().count() {
        x if x < 2 => 65,
        x if x < 5 => 100,
        x if x < 7 => 135,
        _ => 170,
    };

    render!( rect {
        width: "300",
        height: "{height}",
        background: "rgb(60, 60, 60)",
        corner_radius: "8",
        layer: "-50",
        padding: "10",
        shadow: "0 5 10 0 rgb(0, 0, 0, 50)",
        border: "1 solid rgb(50, 50, 50)",
        ScrollView {
            label {
                width: "100%",
                color: "rgb(245, 245, 245)",
                "{content}"
            }
        }
    })
}
