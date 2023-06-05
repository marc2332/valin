use crate::TextArea;
use freya::prelude::*;

pub struct Command {
    name: String,
    run: Box<dyn Fn(&str)>,
}

impl Command {
    pub fn new(name: String, run: Box<dyn Fn(&str)>) -> Self {
        Self { name, run }
    }

    pub fn run(&self, args: &str) {
        (self.run)(args);
    }
}

#[allow(non_snake_case)]
#[inline_props]
pub fn Commander<'a>(
    cx: Scope<'a>,
    commands: &'a Vec<Command>,
    onsubmit: EventHandler<'a>,
) -> Element<'a> {
    let value = use_state(cx, String::new);

    let onsubmit = |new_value: String| {
        let sep = new_value.find(' ');
        if let Some(sep) = sep {
            let (name, args) = new_value.split_at(sep);
            let command = commands.iter().find(|c| c.name == name);
            if let Some(command) = command {
                command.run(args.trim());
                value.set("".to_string());
                onsubmit.call(());
            }
        }
    };

    render!(
        rect {
            width: "100%",
            height: "0",
            layer: "-100",
            scroll_y: "50",
            container {
                width: "100%",
                height: "100",
                display: "center",
                direction: "both",
                padding: "10",
                rect {
                    background: "rgb(20, 20, 20)",
                    shadow: "0 10 40 15 black",
                    radius: "12",
                    onmousedown: |_| {},
                    width: "300",
                    padding: "18",
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
