use std::path::PathBuf;

use crate::controlled_virtual_scroll_view::*;
use crate::lsp::LspConfig;
use crate::manager::EditorManager;
use crate::parser::SyntaxBlocks;
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
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::StreamExt;
use winit::window::CursorIcon;

#[derive(Props, PartialEq)]
pub struct EditorProps {
    pub manager: UseState<EditorManager>,
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
    let manager = cx.props.manager.clone();
    let editor = cx
        .props
        .manager
        .panel(cx.props.panel_index)
        .tab(cx.props.editor)
        .as_text_editor()
        .unwrap();
    let path = editor.path();
    let file_uri = Url::from_file_path(path).unwrap();
    let cursor = editor.cursor();
    let edit_trigger = use_ref(cx, || {
        let (tx, rx) = unbounded_channel::<()>();
        (tx, Some(rx))
    });
    let editable = use_edit(
        cx,
        &manager,
        cx.props.panel_index,
        cx.props.editor,
        edit_trigger.read().0.clone(),
    );
    let hover = use_ref(cx, || None);
    let metrics = use_metrics(
        cx,
        &manager,
        cx.props.panel_index,
        cx.props.editor,
        edit_trigger,
    );
    let cursor_coords = use_ref::<Option<CursorPoint>>(cx, || None);
    let offset_x = use_state(cx, || 0);
    let offset_y = use_state(cx, || 0);

    let cursor_attr = editable.cursor_attr(cx);
    let font_size = manager.font_size();
    let manual_line_height = manager.font_size() * manager.line_height();
    let is_panel_focused = manager.focused_panel() == cx.props.panel_index;
    let is_editor_focused = manager.is_focused()
        && manager.panel(cx.props.panel_index).active_tab() == Some(cx.props.editor);
    let lsp_config = LspConfig::new(cx.props.root_path.clone(), &cx.props.language_id);

    // Trigger initial highlighting
    use_effect(cx, (), move |_| {
        edit_trigger.read().0.send(()).ok();
        async move {}
    });

    let lsp_actions = use_coroutine(cx, |mut rx: UnboundedReceiver<LspAction>| {
        to_owned![lsp_config, hover, manager];
        async move {
            while let Some(action) = rx.next().await {
                match action {
                    LspAction::Hover(params) => {
                        let manager = manager.current();
                        let lsp = manager.lsp(&lsp_config);

                        if let Some(mut lsp) = lsp.cloned() {
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
                                println!("still indexing...");
                            }
                        } else {
                            println!("Lsp not running...");
                        }
                    }
                    LspAction::Clear => {
                        *hover.write() = None;
                    }
                }
            }
        }
    });

    use_effect(cx, (), {
        to_owned![lsp_config, file_uri, manager];
        move |_| {
            // Focus editor
            manager.with_mut(|manager| {
                manager.set_focused_panel(cx.props.panel_index);
                manager
                    .panel_mut(cx.props.panel_index)
                    .set_active_tab(cx.props.editor);
            });

            // Connect to LSP
            let text = editor.rope().to_string();
            async move {
                let mut lsp = EditorManager::get_or_insert_lsp(manager, &lsp_config).await;

                lsp.server_socket
                    .did_open(DidOpenTextDocumentParams {
                        text_document: TextDocumentItem {
                            uri: file_uri.clone(),
                            language_id: "rust".into(),
                            version: 0,
                            text,
                        },
                    })
                    .unwrap();
            }
        }
    });

    let onmousedown = {
        to_owned![manager];
        move |_: MouseEvent| {
            if !is_editor_focused {
                manager.with_mut(|manager| {
                    manager.set_focused_panel(cx.props.panel_index);
                    manager
                        .panel_mut(cx.props.panel_index)
                        .set_active_tab(cx.props.editor);
                });
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
                width: "100%",
                height: "100%",
                show_scrollbar: true,
                builder_values: (cursor, metrics.clone(), editable, lsp_actions.clone(), file_uri, editor.rope(), hover.clone(), cursor_coords.clone()),
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
    use_editable::UseEditable,
    Coroutine<LspAction>,
    Url,
    &'a Rope,
    UseRef<Option<(u32, Hover)>>,
    UseRef<Option<CursorPoint>>,
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
    let (cursor, metrics, editable, lsp_actions, file_uri, rope, hover, cursor_coords) = options;
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

    let onmouseover = {
        to_owned![editable, file_uri, lsp_actions, cursor_coords, hover];
        move |e: MouseEvent| {
            let coords = e.get_element_coordinates();
            editable.process_event(&EditableEvent::MouseOver(e.data, *line_index));

            if hover.read().is_some() {
                *cursor_coords.write() = Some(coords);
            }

            let paragraph = create_paragraph(&line_str, *font_size);

            if (coords.x as f32) < paragraph.max_intrinsic_width() {
                let glyph =
                    paragraph.get_glyph_position_at_coordinate((coords.x as i32, coords.y as i32));

                lsp_actions.send(LspAction::Hover(HoverParams {
                    text_document_position_params: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier {
                            uri: file_uri.clone(),
                        },
                        position: Position::new(*line_index as u32, glyph.position as u32),
                    },
                    work_done_progress_params: WorkDoneProgressParams::default(),
                }));
            } else {
                lsp_actions.send(LspAction::Clear);
            }
        }
    };

    let gutter_width = font_size * 3.0;

    render!(
        if let Some((line, hover)) = hover.read().as_ref() {
            if *line == *line_index as u32 {
                if let Some(content) = hover.hover_to_text() {
                    let cursor_coords = cursor_coords.read();
                    if let Some(cursor_coords) = cursor_coords.as_ref().cloned() {
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
                    } else {
                        None
                    }
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
            CursorArea {
                icon: CursorIcon::Text,
                paragraph {
                    min_width: "calc(100% - {gutter_width)",
                    width: "{width}",
                    cursor_index: "{character_index}",
                    cursor_color: "white",
                    max_lines: "1",
                    cursor_mode: "editable",
                    cursor_id: "{line_index}",
                    onmousedown: onmousedown,
                    onmouseover: onmouseover,
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
