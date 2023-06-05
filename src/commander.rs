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
        container {
            width: "100%",
            height: "200",
            display: "center",
            direction: "vertical",
            background: "rgb(20, 20, 20)",
            TextArea {
                value: "{value}",
                onchange: |v| value.set(v),
                onsubmit: onsubmit,
            }
        }
    )
}
