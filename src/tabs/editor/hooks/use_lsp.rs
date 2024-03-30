use async_lsp::LanguageServer;
use freya::prelude::*;
use lsp_types::{DidOpenTextDocumentParams, Hover, HoverParams, TextDocumentItem, Url};
use tokio_stream::StreamExt;

use crate::{
    hooks::{EditorManager, UseManager},
    lsp::{LanguageId, LspConfig},
};

#[derive(Clone, PartialEq)]
pub enum LspAction {
    Hover(HoverParams),
    Clear,
}

#[derive(Clone, PartialEq)]
pub struct UseLsp {
    lsp_coroutine: Coroutine<LspAction>,
}

impl UseLsp {
    pub fn send(&self, action: LspAction) {
        self.lsp_coroutine.send(action)
    }
}

pub fn use_lsp(
    language_id: LanguageId,
    panel_index: usize,
    editor_index: usize,
    lsp_config: &Option<LspConfig>,
    manager: &UseManager,
    hover_location: &Signal<Option<(u32, Hover)>>,
) -> UseLsp {
    use_hook(|| {
        to_owned![lsp_config, manager];
        let language_id = language_id.to_string();

        if let Some(lsp_config) = lsp_config {
            let (file_uri, file_text) = {
                let manager = manager.current();

                let editor = manager
                    .panel(panel_index)
                    .tab(editor_index)
                    .as_text_editor()
                    .unwrap();

                let path = editor.path();
                (
                    Url::from_file_path(path).unwrap(),
                    editor.rope().to_string(),
                )
            };

            // Notify language server the file has been opened
            spawn(async move {
                let mut lsp = EditorManager::get_or_insert_lsp(manager, &lsp_config).await;

                lsp.server_socket
                    .did_open(DidOpenTextDocumentParams {
                        text_document: TextDocumentItem {
                            uri: file_uri,
                            language_id,
                            version: 0,
                            text: file_text,
                        },
                    })
                    .unwrap();
            });
        }
    });

    let lsp_coroutine = use_coroutine(|mut rx: UnboundedReceiver<LspAction>| {
        to_owned![lsp_config, hover_location, manager];
        async move {
            if let Some(lsp_config) = lsp_config {
                while let Some(action) = rx.next().await {
                    match action {
                        LspAction::Hover(params) => {
                            let lsp = manager.current().lsp(&lsp_config).cloned();

                            if let Some(mut lsp) = lsp {
                                let is_indexed = *lsp.indexed.lock().unwrap();
                                if is_indexed {
                                    let line = params.text_document_position_params.position.line;
                                    let response = lsp.server_socket.hover(params).await;

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

    UseLsp {
        lsp_coroutine: lsp_coroutine.clone(),
    }
}
