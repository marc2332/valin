use std::collections::HashMap;

pub trait EditorCommand {
    fn is_visible(&self) -> bool {
        true
    }

    fn matches(&self, input: &str) -> bool;

    fn id(&self) -> &str;

    fn text(&self) -> &str;

    fn run(&self);
}

#[derive(Default)]
pub struct EditorCommands {
    pub(crate) commands: HashMap<String, Box<dyn EditorCommand>>,
}

impl EditorCommands {
    pub fn register(&mut self, editor: impl EditorCommand + 'static) {
        self.commands
            .insert(editor.id().to_string(), Box::new(editor));
    }

    pub fn trigger(&self, command_name: &str) {
        let command = self.commands.get(command_name);

        if let Some(command) = command {
            command.run();
        }
    }
}
