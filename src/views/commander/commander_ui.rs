use crate::{
    components::Overlay,
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
        let value = use_state(String::new);
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

        use_side_effect(move || {
            let _ = value.read();
            selected.set_if_modified(0);
        });

        let selected_index = *selected.read();

        Overlay::new().child(
            rect()
                .on_key_down(onkeydown)
                .spacing(5.)
                .child(
                    Input::new(value)
                        .width(Size::fill())
                        .auto_focus(true)
                        .inner_margin(12.)
                        .placeholder("Run a command...")
                        .on_submit(on_submit)
                        .on_pre_key_down(|e: Event<KeyboardEventData>| match e.code {
                            Code::ArrowUp | Code::ArrowDown => false,
                            _ => match &e.key {
                                Key::Named(NamedKey::Enter) | Key::Named(NamedKey::Escape) => true,
                                Key::Named(NamedKey::Tab) => false,
                                _ => {
                                    e.stop_propagation();
                                    e.prevent_default();
                                    true
                                }
                            },
                        }),
                )
                .child(
                    ScrollView::new()
                        .height(Size::px(options_height as f32))
                        .child(if filtered_commands.is_empty() {
                            CommanderOption {
                                command_id: "not-found".to_string(),
                                command_text: "Command Not Found".to_string(),
                                is_selected: true,
                            }
                            .into_element()
                        } else {
                            rect()
                                .children(filtered_commands.into_iter().enumerate().map(
                                    |(n, command_id)| {
                                        let command = commands.commands.get(&command_id).unwrap();
                                        CommanderOption {
                                            command_id: command_id.clone(),
                                            command_text: command.text().to_string(),
                                            is_selected: n == selected_index,
                                        }
                                        .into()
                                    },
                                ))
                                .into_element()
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
    fn render_key(&self) -> DiffKey {
        DiffKey::from(&self.command_id)
    }

    fn render(&self) -> impl IntoElement {
        let background = if self.is_selected {
            Color::from((22, 27, 34))
        } else {
            Color::TRANSPARENT
        };

        rect()
            .background(background)
            .padding((8., 6.))
            .width(Size::fill())
            .height(Size::px(30.))
            .corner_radius(10.)
            .main_align(Alignment::Center)
            .child(self.command_text.clone())
    }
}
