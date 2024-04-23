use freya::prelude::*;
use lsp_types::{
    Hover, HoverParams, Position, TextDocumentIdentifier, TextDocumentPositionParams, Url,
    WorkDoneProgressParams,
};
use std::sync::Arc;
use tokio_stream::StreamExt;
use tracing::info;

use crate::{
    lsp::LspConfig,
    state::{AppState, RadioAppState},
    Args, EditorType,
};

#[derive(Clone, PartialEq)]
pub enum LspAction {
    Hover(Position),
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
    editor_type: EditorType,
    panel_index: usize,
    editor_index: usize,
    radio: RadioAppState,
    mut hover_location: Signal<Option<(u32, Hover)>>,
) -> UseLsp {
    let args = use_context::<Arc<Args>>();
    let lsp_config = args.lsp.then(|| LspConfig::new(editor_type)).flatten();

    use_hook(|| {
        to_owned![lsp_config];

        if let Some(lsp_config) = lsp_config {
            let (file_uri, file_text) = {
                let app_state = radio.read();
                let editor = app_state.editor(panel_index, editor_index);
                (editor.uri(), editor.text())
            };

            if let Some(file_uri) = file_uri {
                // Notify language server the file has been opened
                spawn(async move {
                    let mut lsp = AppState::get_or_insert_lsp(radio, &lsp_config).await;
                    lsp.open_file(file_uri, file_text);
                });
            }
        }
    });

    let lsp_coroutine = use_coroutine(|mut rx: UnboundedReceiver<LspAction>| {
        to_owned![lsp_config];
        async move {
            if let Some(lsp_config) = lsp_config {
                let (file_path, _) = lsp_config
                    .editor_type
                    .paths()
                    .expect("Something went wrong.");
                let file_uri = Url::from_file_path(file_path).unwrap();

                while let Some(action) = rx.next().await {
                    let lsp = radio.read().lsp(&lsp_config).cloned();
                    let mut lsp = if let Some(lsp) = lsp {
                        let is_indexed = *lsp.indexed.lock().unwrap();
                        if is_indexed {
                            lsp
                        } else {
                            info!("Language Server is indexing.");
                            continue;
                        }
                    } else {
                        info!("Language Server not running.");
                        continue;
                    };

                    match action {
                        LspAction::Hover(position) => {
                            let line = position.line;
                            let response = lsp
                                .hover_file_with_prams(HoverParams {
                                    text_document_position_params: TextDocumentPositionParams {
                                        text_document: TextDocumentIdentifier {
                                            uri: file_uri.clone(),
                                        },
                                        position,
                                    },
                                    work_done_progress_params: WorkDoneProgressParams::default(),
                                })
                                .await;

                            if let Ok(Some(res)) = response {
                                *hover_location.write() = Some((line, res));
                            } else {
                                *hover_location.write() = None;
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
