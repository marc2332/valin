use crate::{
    keyboard_navigation::use_keyboard_navigation,
    state::{Channel, EditorCommands, EditorView, RadioAppState},
    TextArea,
};
use freya::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct CommanderProps {
    editor_commands: Signal<EditorCommands>,
    radio_app_state: RadioAppState
}

#[allow(non_snake_case)]
pub fn Commander(CommanderProps { editor_commands, mut radio_app_state }: CommanderProps) -> Element {
    let mut value = use_signal(String::new);
    let mut selected = use_signal(|| 0);
    let mut keyboard_navigation = use_keyboard_navigation();
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
    let options_height = ((filtered_commands_len.max(1)) * 30).min(200);

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
            // Run the command
            command.run();

            // Focus the previous view
            keyboard_navigation.callback(true, move || {
                let mut app_state = radio_app_state.write();
                app_state.set_focused_view_to_previous();
            })
        }
    };

    let onkeydown = move |e: KeyboardEvent| {
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

    let onglobalmousedown = move |_| {
        if *radio_app_state.read().focused_view() == EditorView::Commander {
            let mut app_state = radio_app_state.write_channel(Channel::Global);
            app_state.set_focused_view_to_previous();
        }
    };

    rsx!(
        rect {
            width: "100%",
            height: "0",
            layer: "-100",
            onglobalmousedown,
            onkeydown,
            rect {
                width: "100%",
                main_align: "center",
                cross_align: "center",
                padding: "10",
                rect {
                    background: "rgb(45, 45, 45)",
                    shadow: "0 4 15 8 rgb(0, 0, 0, 0.3)",
                    corner_radius: "10",
                    onmousedown: |_| {},
                    width: "300",
                    padding: "5",
                    TextArea {
                        value: "{value}",
                        onchange,
                        onsubmit,
                    }
                    ScrollView {
                        theme: theme_with!(ScrollViewTheme {
                            height: options_height.to_string().into(),
                        }),
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
        }
    )
}

fn commander_option(command_id: &str, command_text: &str, is_selected: bool) -> Element {
    let background = if is_selected { "rgb(65, 65, 65)" } else { "" };

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
