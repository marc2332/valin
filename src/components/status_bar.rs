use freya::prelude::*;

use crate::{state::EditorView, LspStatuses};

#[derive(Props, Clone, PartialEq)]
pub struct StatusBarProps {
    #[props(!optional)]
    cursor: Option<TextCursor>,
    lsp_statuses: LspStatuses,
    focused_view: EditorView,
}

#[allow(non_snake_case)]
pub fn StatusBar(props: StatusBarProps) -> Element {
    rsx!(
        rect {
            width: "100%",
            height: "fill",
            background: "rgb(20, 20, 20)",
            direction: "horizontal",
            cross_align: "center",
            padding: "0 6",
            color: "rgb(220, 220, 220)",
            label {
                "{props.focused_view}"
            }
            if let Some(cursor) = props.cursor {
                label {
                    " | Ln {cursor.row() + 1}, Col {cursor.col() + 1}"
                }
            }
            for (name, msg) in props.lsp_statuses.read().iter() {
                label {
                    " | {name} {msg}"
                }
            }
        }
    )
}
