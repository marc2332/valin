use freya::prelude::*;
use lsp_types::{Hover, HoverParams};
use std::path::PathBuf;
use tokio_stream::StreamExt;

use crate::{
    lsp::{LanguageId, LspConfig},
    state::{AppState, RadioAppState},
};

#[derive(Clone, PartialEq)]
pub enum LspAction {
    Hover(HoverParams),
    Clear,
}

#[derive(Clone, PartialEq, Copy)]
pub struct UseLsp {
    lsp_coroutine: Coroutine<LspAction>,
}

impl UseLsp {
    pub fn send(&self, action: LspAction) {
        self.lsp_coroutine.send(action)
    }
}

pub fn use_lsp(
    root_path: PathBuf,
    language_id: LanguageId,
    panel_index: usize,
    editor_index: usize,
    radio: RadioAppState,
    mut hover_location: Signal<Option<(u32, Hover)>>,
) -> UseLsp {
    let lsp_config = LspConfig::new(root_path, language_id);

    use_hook(|| {
        to_owned![lsp_config];

        if let Some(lsp_config) = lsp_config {
            let (file_uri, file_text) = {
                let app_state = radio.read();
                let editor = app_state.editor(panel_index, editor_index);
                (editor.uri(), editor.text())
            };

            // Notify language server the file has been opened
            spawn(async move {
                let mut lsp = AppState::get_or_insert_lsp(radio, &lsp_config).await;

                lsp.open_file(language_id, file_uri, file_text);
            });
        }
    });

    let lsp_coroutine = use_coroutine(|mut rx: UnboundedReceiver<LspAction>| {
        to_owned![lsp_config];
        async move {
            if let Some(lsp_config) = lsp_config {
                while let Some(action) = rx.next().await {
                    match action {
                        LspAction::Hover(params) => {
                            let lsp = radio.read().lsp(&lsp_config).cloned();

                            if let Some(mut lsp) = lsp {
                                let is_indexed = *lsp.indexed.lock().unwrap();
                                if is_indexed {
                                    let line = params.text_document_position_params.position.line;
                                    let response = lsp.hover_file_with_prams(params).await;

                                    if let Ok(Some(res)) = response {
                                        *hover_location.write() = Some((line, res));
                                    } else {
                                        *hover_location.write() = None;
                                    }
                                } else {
                                    println!("LSP: Still indexing...");
                                }
                            } else {
                                println!("LSP: Not running.");
                            }
                        }
                        LspAction::Clear => {
                            *hover_location.write() = None;
                        }
                    }
                }
            }
        }
    });

    UseLsp { lsp_coroutine }
}
