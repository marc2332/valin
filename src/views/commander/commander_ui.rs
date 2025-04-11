use crate::{
    state::{Channel, CommandRunContext, EditorCommands},
    Overlay, TextArea,
};
use dioxus_radio::prelude::use_radio;
use freya::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct CommanderProps {
    editor_commands: Signal<EditorCommands>,
}

#[allow(non_snake_case)]
pub fn Commander(CommanderProps { editor_commands }: CommanderProps) -> Element {
    let mut radio_app_state = use_radio(Channel::Global);
    let mut value = use_signal(String::new);
    let mut selected = use_signal(|| 0);
    let mut focus = use_focus();

    let commands = editor_commands.read();
    let filtered_commands = commands
        .commands
        .iter()
        .filter_map(|(id, command)| {
            if value.read().is_empty() {
                command.is_visible()
            } else {
                command.is_visible() && command.matches(value.read().as_str())
            }
            .then_some(id.clone())
        })
        .collect::<Vec<String>>();
    let filtered_commands_len = filtered_commands.len();
    let options_height = ((filtered_commands_len.max(1)) * 30).max(175);

    let onchange = move |v| {
        if *value.read() != v {
            selected.set(0);
            value.set(v);
        }
    };

    let command_id = filtered_commands.get(selected()).cloned();

    let onsubmit = move |_: String| {
        let editor_commands = editor_commands.read();
        let command = command_id
            .as_ref()
            .and_then(|command_i| editor_commands.commands.get(command_i));
        if let Some(command) = command {
            let mut ctx = CommandRunContext::default();

            // Run the command
            command.run(&mut ctx);

            if ctx.focus_previous_view {
                let mut app_state = radio_app_state.write();
                app_state.focus_previous_view();
            }
        }
    };

    let onkeydown = move |e: KeyboardEvent| {
        e.stop_propagation();
        focus.prevent_navigation();
        match e.code {
            Code::ArrowDown => {
                if filtered_commands_len > 0 {
                    if *selected.read() < filtered_commands_len - 1 {
                        *selected.write() += 1;
                    } else {
                        selected.set(0);
                    }
                }
            }
            Code::ArrowUp => {
                if selected() > 0 && filtered_commands_len > 0 {
                    *selected.write() -= 1;
                } else {
                    selected.set(filtered_commands_len - 1);
                }
            }
            _ => {}
        }
    };

    rsx!(
        Overlay {
            rect {
                onkeydown,
                spacing: "5",
                TextArea {
                    placeholder: "Run a command...",
                    value: "{value}",
                    onchange,
                    onsubmit,
                }
                ScrollView {
                    height: "{options_height}",
                    if filtered_commands.is_empty() {
                        {commander_option("not-found", "Command Not Found", true)}
                    }
                    for (n, command_id) in filtered_commands.into_iter().enumerate() {
                        {
                            let command = commands.commands.get(&command_id).unwrap();
                            commander_option(&command_id, command.text(), n == selected())
                        }
                    }
                }
            }
        }
    )
}

fn commander_option(command_id: &str, command_text: &str, is_selected: bool) -> Element {
    let background = if is_selected {
        "rgb(29, 32, 33)"
    } else {
        "none"
    };

    rsx!(
        rect {
            background,
            key: "{command_id}",
            padding: "8 6",
            width: "100%",
            height: "30",
            corner_radius: "10",
            main_align: "center",
            label {
                "{command_text}"
            }
        }
    )
}
