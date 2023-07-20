use crate::controlled_virtual_scroll_view::*;
use crate::panels::PanelsManager;
use crate::use_editable::*;
use crate::use_metrics::*;
use freya::prelude::events::KeyboardEvent;
use freya::prelude::*;
use tokio::sync::mpsc::unbounded_channel;
use winit::window::CursorIcon;

#[derive(Props, PartialEq)]
pub struct EditorProps {
    pub manager: UseState<PanelsManager>,
    pub panel_index: usize,
    pub editor: usize,
}

#[allow(non_snake_case)]
pub fn CodeEditorTab(cx: Scope<EditorProps>) -> Element {
    let editor = cx
        .props
        .manager
        .panel(cx.props.panel_index)
        .tab(cx.props.editor)
        .as_text_editor()
        .unwrap();
    let cursor = editor.cursor();
    let edit_trigger = use_ref(cx, || {
        let (tx, rx) = unbounded_channel::<()>();
        (tx, Some(rx))
    });
    let editable = use_edit(
        cx,
        &cx.props.manager,
        cx.props.panel_index,
        cx.props.editor,
        edit_trigger.read().0.clone(),
    );

    // Trigger initial highlighting
    use_effect(cx, (), move |_| {
        edit_trigger.read().0.send(()).ok();
        async move {}
    });

    let metrics = use_metrics(
        cx,
        &cx.props.manager,
        cx.props.panel_index,
        cx.props.editor,
        edit_trigger,
    );
    let offset_x = use_state(cx, || 0);
    let offset_y = use_state(cx, || 0);

    let cursor_attr = editable.cursor_attr(cx);
    let font_size = cx.props.manager.font_size();
    let manual_line_height = cx.props.manager.font_size() * cx.props.manager.line_height();
    let is_panel_focused = cx.props.manager.focused_panel() == cx.props.panel_index;
    let is_editor_focused = cx.props.manager.is_focused()
        && cx.props.manager.panel(cx.props.panel_index).active_tab() == Some(cx.props.editor);

    let onmousedown = move |_: MouseEvent| {
        if !is_editor_focused {
            cx.props.manager.with_mut(|manager| {
                manager.set_focused_panel(cx.props.panel_index);
                manager
                    .panel_mut(cx.props.panel_index)
                    .set_active_tab(cx.props.editor);
            });
        }
    };

    let onscroll = move |(axis, scroll): (Axis, i32)| match axis {
        Axis::Y => offset_y.set(scroll),
        Axis::X => offset_x.set(scroll),
    };

    use_effect(cx, (), move |_| {
        cx.props.manager.with_mut(|manager| {
            manager.set_focused_panel(cx.props.panel_index);
            manager
                .panel_mut(cx.props.panel_index)
                .set_active_tab(cx.props.editor);
        });
        async move {}
    });

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
                builder_values: (cursor.clone(), metrics, editable),
                length: metrics.0.len(),
                item_size: manual_line_height,
                builder: Box::new(move |(k, line_index, cx, args)| {
                    let (cursor, metrics, editable) = args.as_ref().unwrap();
                    let (syntax_blocks, width) = metrics.get();
                    let line = syntax_blocks.get(line_index).unwrap();
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
                        to_owned![editable];
                        move |e: MouseEvent| {
                            editable.process_event(&EditableEvent::MouseOver(e.data, line_index));
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
