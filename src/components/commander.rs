use crate::{commands::EditorCommand, TextArea};
use freya::prelude::*;

#[derive(Props)]
pub struct CommanderProps<'a> {
    commands: &'a Vec<Box<dyn EditorCommand>>,
    onsubmit: EventHandler<'a>,
}

#[allow(non_snake_case)]
pub fn Commander<'a>(cx: Scope<'a, CommanderProps<'a>>) -> Element<'a> {
    let value = use_state(cx, String::new);

    let onsubmit = |new_value: String| {
        let sep = new_value.find(' ');
        if let Some(sep) = sep {
            let (name, args) = new_value.split_at(sep);
            let command = cx.props.commands.iter().find(|c| c.name() == name);
            if let Some(command) = command {
                command.run_with_args(args.trim());
                value.set("".to_string());
                cx.props.onsubmit.call(());
            }
        }
    };

    render!(
        rect {
            width: "100%",
            height: "0",
            layer: "-100",
            rect {
                width: "100%",
                main_align: "center",
                cross_align: "center",
                padding: "10",
                rect {
                    background: "rgb(45, 45, 45)",
                    shadow: "0 2 20 5 rgb(0, 0, 0, 100)",
                    corner_radius: "10",
                    onmousedown: |_| {},
                    width: "300",
                    padding: "5",
                    TextArea {
                        value: "{value}",
                        onchange: |v| value.set(v),
                        onsubmit: onsubmit,
                    }
                }
            }
        }
    )
}
