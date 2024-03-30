use std::collections::HashMap;

use freya::prelude::*;

use crate::hooks::EditorView;

#[derive(Props, Clone, PartialEq)]
pub struct StatusBarProps {
    #[props(!optional)]
    cursor: Option<TextCursor>,
    lsp_messages: Signal<HashMap<String, String>>,
    focused_view: EditorView,
}

#[allow(non_snake_case)]
pub fn StatusBar(props: StatusBarProps) -> Element {
    rsx!(
        rect {
            width: "100%",
            height: "25",
            background: "rgb(20, 20, 20)",
            direction: "horizontal",
            cross_align: "center",
            padding: "0 6",
            color: "rgb(200, 200, 200)",
            label {
                font_size: "14",
                "{props.focused_view}"
            }
            if let Some(cursor) = props.cursor {
                label {
                    font_size: "14",
                    " | Ln {cursor.row() + 1}, Col {cursor.col() + 1}"
                }
            }
            for (name, msg) in props.lsp_messages.read().iter() {
                label {
                    font_size: "14",
                    " | {name} {msg}"
                }
            }
        }
    )
}
