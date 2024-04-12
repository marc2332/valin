use std::rc::Rc;

use crate::{commands::EditorCommand, TextArea};
use freya::prelude::*;

#[derive(Props, Clone)]
pub struct CommanderProps {
    commands: Rc<Vec<Box<dyn EditorCommand>>>,
    onsubmit: EventHandler,
}

impl PartialEq for CommanderProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.commands, &other.commands) && self.onsubmit == other.onsubmit
    }
}

#[allow(non_snake_case)]
pub fn Commander(props: CommanderProps) -> Element {
    let mut value = use_signal(String::new);

    let onsubmit = move |new_value: String| {
        let sep = new_value.find(' ');
        if let Some(sep) = sep {
            let (name, args) = new_value.split_at(sep);
            let command = props.commands.iter().find(|c| c.name() == name);
            if let Some(command) = command {
                command.run_with_args(args.trim());
                value.set("".to_string());
                props.onsubmit.call(());
            }
        }
    };

    rsx!(
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
                        onchange: move |v| value.set(v),
                        onsubmit,
                    }
                }
            }
        }
    )
}
