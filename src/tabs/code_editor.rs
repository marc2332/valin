use std::path::PathBuf;

use crate::controlled_virtual_scroll_view::*;
use crate::lsp::LspConfig;
use crate::panels::PanelsManager;
use crate::use_editable::*;
use crate::use_metrics::*;
use async_lsp::LanguageServer;
use freya::prelude::events::KeyboardEvent;
use freya::prelude::*;
use lsp_types::Hover;
use lsp_types::MarkedString;

use lsp_types::{
    DidOpenTextDocumentParams, HoverContents, HoverParams, Position, TextDocumentIdentifier,
    TextDocumentItem, TextDocumentPositionParams, Url, WorkDoneProgressParams,
};
use skia_safe::scalar;
use skia_safe::textlayout::FontCollection;
use skia_safe::textlayout::ParagraphBuilder;
use skia_safe::textlayout::ParagraphStyle;
use skia_safe::textlayout::TextStyle;
use skia_safe::FontMgr;
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::StreamExt;
use winit::window::CursorIcon;

#[derive(Props, PartialEq)]
pub struct EditorProps {
    pub manager: UseState<PanelsManager>,
    pub panel_index: usize,
    pub editor: usize,
    pub language_id: String,
    pub root_path: PathBuf,
}

pub enum LspAction {
    Hover(HoverParams),
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
    let cursor_coords = use_ref(cx, || None);
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
                                    println!("{res:?}");
                                    *hover.write() = Some((line, res));
                                }
                            } else {
                                println!("still indexing...");
                            }
                        } else {
                            println!("Lsp not running...");
                        }
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
                let mut lsp = PanelsManager::get_or_insert_lsp(manager, &lsp_config).await;

                lsp.server_socket
                    .did_open(DidOpenTextDocumentParams {
                        text_document: TextDocumentItem {
                            uri: file_uri.clone(),
                            language_id: "rust".into(),
                            version: 0,
                            text: text.into(),
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
            height: "calc(100% - 30.0)",
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
                builder_values: (cursor.clone(), metrics, editable, lsp_actions.clone(), file_uri.clone(), editor.rope(), hover.clone(), cursor_coords.clone()),
                length: metrics.0.len(),
                item_size: manual_line_height,
                builder: Box::new(move |(k, line_index, cx, args)| {
                    let (cursor, metrics, editable, lsp_actions, file_uri, rope, hover, cursor_coords) = args.as_ref().unwrap();
                    let (syntax_blocks, width) = metrics.get();
                    let line = syntax_blocks.get(line_index).unwrap();
                    let line_str = rope.line(line_index).to_string();
                    let highlights_attr = editable.highlights_attr(cx, line_index);

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

                    let onmousedown = {
                        to_owned![editable];
                        move |e: MouseEvent| {
                            editable.process_event(&EditableEvent::MouseDown(e.data, line_index));
                        }
                    };

                    let onmouseover = {
                        to_owned![editable, file_uri, lsp_actions, cursor_coords, hover];
                        move |e: MouseEvent| {
                            let coords = e.get_element_coordinates();
                            editable.process_event(&EditableEvent::MouseOver(e.data, line_index));

                            if hover.read().is_some() {
                                *cursor_coords.write() = Some(coords);
                            }

                            let mut font_collection = FontCollection::new();
                            font_collection.set_default_font_manager(FontMgr::default(), "Jetbrains Mono");

                            let mut style = ParagraphStyle::default();
                            let mut text_style = TextStyle::default();
                            text_style.set_font_size(font_size);
                            style.set_text_style(&text_style);

                            let mut paragraph = ParagraphBuilder::new(&style, font_collection);

                            paragraph.add_text(line_str.clone());

                            let mut p = paragraph.build();

                            p.layout(scalar::MAX);

                            let glyph = p.get_glyph_position_at_coordinate((coords.x as i32, coords.y  as i32));

                            let pos = glyph.position;

                            lsp_actions.send(LspAction::Hover(HoverParams {
                                text_document_position_params: TextDocumentPositionParams {
                                    text_document: TextDocumentIdentifier { uri: file_uri.clone() },
                                    position: Position::new(line_index as u32, pos as u32),
                                },
                                work_done_progress_params: WorkDoneProgressParams::default(),
                            }));

                        }
                    };

                    rsx!(
                        rect {
                            key: "{k}",
                            height: "{manual_line_height}",
                            direction: "horizontal",
                            background: "{line_background}",
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
                            if let Some((line, hover)) = hover.read().as_ref() {
                                if *line == line_index as u32 {
                                    if let Some(content) = hover.hover_to_text() {
                                        let cursor_coords = cursor_coords.read();
                                        if let Some(cursor_coords) = cursor_coords.as_ref().cloned() {
                                            Some(rsx!(
                                                rect {
                                                    width: "0",
                                                    height: "0",
                                                    offset_y: "{manual_line_height}",
                                                    offset_x: "{cursor_coords.x}",
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
                            CursorArea {
                                icon: CursorIcon::Text,
                                paragraph {
                                    min_width: "calc(100% - {font_size * 3.0})",
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
                })
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
    render!( rect {
        width: "300",
        height: "180",
        background: "rgb(60, 60, 60)",
        corner_radius: "8",
        layer: "-50",
        padding: "10",
        ScrollView {
            label {
                width: "100%",
                color: "rgb(245, 245, 245)",
                "{content}"
            }
        }
    })
}
