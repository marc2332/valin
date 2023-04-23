use freya::prelude::*;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::parser::*;
use crate::EditorManager;

pub fn use_syntax_highlighter<'a>(
    cx: &'a ScopeState,
    manager: &UseState<EditorManager>,
    pane_index: usize,
    editor: usize,
    highlight_trigger: &UseRef<(UnboundedSender<()>, Option<UnboundedReceiver<()>>)>,
) -> &'a UseState<SyntaxBlocks> {
    let syntax_blocks = use_state::<SyntaxBlocks>(cx, Vec::new);

    use_effect(cx, (), move |_| {
        let syntax_blocks = syntax_blocks.clone();
        let manager = manager.clone();
        let highlight_receiver = &mut highlight_trigger.write().1;
        let mut highlight_receiver = highlight_receiver.take().unwrap();

        async move {
            while highlight_receiver.recv().await.is_some() {
                let manager = manager.current();
                let editor = &manager.panel(pane_index).editor(editor);

                syntax_blocks.with_mut(|syntax_blocks| parse(editor, syntax_blocks));
            }
        }
    });

    syntax_blocks
}
