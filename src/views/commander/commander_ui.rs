use crate::{
    components::{Overlay, TextArea},
    state::{AppState, Channel, CommandRunContext, EditorCommands},
};
use freya::prelude::*;
use freya::radio::use_radio;

#[derive(PartialEq)]
pub struct Commander {
    pub editor_commands: State<EditorCommands>,
}

impl Component for Commander {
    fn render(&self) -> impl IntoElement {
        let mut radio_app_state = use_radio::<AppState, Channel>(Channel::Global);
        let mut value = use_state(String::new);
        let mut selected = use_state(|| 0usize);

        let editor_commands = self.editor_commands;
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

        let command_id = filtered_commands.get(*selected.read()).cloned();

        let onchange = move |v| {
            if *value.read() != v {
                selected.set(0);
                value.set(v);
            }
        };

        let on_submit = move |_: String| {
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

        let onkeydown = move |e: Event<KeyboardEventData>| {
            e.stop_propagation();
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
                    if *selected.read() > 0 && filtered_commands_len > 0 {
                        *selected.write() -= 1;
                    } else {
                        selected.set(filtered_commands_len - 1);
                    }
                }
                _ => {}
            }
        };

        let selected_index = *selected.read();

        Overlay::new().child(
            rect()
                .on_key_down(onkeydown)
                .spacing(5.)
                .child(
                    TextArea::new()
                        .placeholder("Run a command...")
                        .value(value.read().clone())
                        .onchange(onchange)
                        .on_submit(on_submit),
                )
                .child(
                    ScrollView::new()
                        .height(Size::px(options_height as f32))
                        .child({
                            let content: Element = if filtered_commands.is_empty() {
                                CommanderOption {
                                    command_id: "not-found".to_string(),
                                    command_text: "Command Not Found".to_string(),
                                    is_selected: true,
                                }
                                .into()
                            } else {
                                rect()
                                    .children(filtered_commands.into_iter().enumerate().map(
                                        |(n, command_id)| {
                                            let command =
                                                commands.commands.get(&command_id).unwrap();
                                            CommanderOption {
                                                command_id: command_id.clone(),
                                                command_text: command.text().to_string(),
                                                is_selected: n == selected_index,
                                            }
                                            .into()
                                        },
                                    ))
                                    .into()
                            };
                            content
                        }),
                ),
        )
    }
}

#[derive(PartialEq)]
struct CommanderOption {
    command_id: String,
    command_text: String,
    is_selected: bool,
}

impl Component for CommanderOption {
    fn render(&self) -> impl IntoElement {
        let background = if self.is_selected {
            Color::from((29, 32, 33))
        } else {
            Color::TRANSPARENT
        };

        let command_text = self.command_text.clone();
        let command_id = self.command_id.clone();

        rect()
            .background(background)
            .key(&command_id)
            .padding((8., 6.))
            .width(Size::fill())
            .height(Size::px(30.))
            .corner_radius(10.)
            .main_align(Alignment::Center)
            .child(label().text(command_text))
    }
}
