use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::{fmt::Display, ops::ControlFlow};

use async_lsp::concurrency::ConcurrencyLayer;
use async_lsp::panic::CatchUnwindLayer;
use async_lsp::router::Router;
use async_lsp::tracing::TracingLayer;
use async_lsp::{LanguageServer, ServerSocket};
use async_process::Command;
use lsp_types::{
    notification::{Progress, PublishDiagnostics, ShowMessage},
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, HoverParams, TextDocumentIdentifier,
    TextDocumentItem,
};
use lsp_types::{
    ClientCapabilities, InitializeParams, InitializedParams, NumberOrString, ProgressParamsValue,
    Url, WindowClientCapabilities, WorkDoneProgress,
};
use tower::ServiceBuilder;
use tracing::info;

use crate::{state::EditorType, LspStatusSender};

struct RouterState {
    pub(crate) indexed: Arc<Mutex<bool>>,
    pub(crate) lsp_sender: LspStatusSender,
    pub(crate) language_server: String,
}

struct Stop;

#[derive(Clone)]
pub struct LSPClient {
    pub(crate) indexed: Arc<Mutex<bool>>,
    pub(crate) server_socket: ServerSocket,
    pub(crate) language_id: LanguageId,
}

impl LSPClient {
    pub fn open_file(&mut self, file_uri: Url, file_text: String) {
        info!(
            "Opened document [uri={file_uri}] from [lsp={:?}]",
            self.language_id
        );
        self.server_socket
            .did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: file_uri,
                    language_id: self.language_id.to_string(),
                    version: 0,
                    text: file_text,
                },
            })
            .unwrap();
    }

    pub fn close_file(&mut self, file_uri: Url) {
        info!("Closed document [uri={file_uri}] from LSP");
        self.server_socket
            .did_close(DidCloseTextDocumentParams {
                text_document: TextDocumentIdentifier { uri: file_uri },
            })
            .unwrap();
    }

    pub async fn hover_file_with_prams(
        &mut self,
        hover_params: HoverParams,
    ) -> Result<Option<lsp_types::Hover>, async_lsp::Error> {
        self.server_socket.hover(hover_params).await
    }
}

#[derive(Clone)]
pub struct LspConfig {
    pub(crate) editor_type: EditorType,
    pub(crate) language_server: String,
}

impl LspConfig {
    pub fn new(editor_type: EditorType) -> Option<Self> {
        let language_server = editor_type.language_id().language_server()?.to_string();

        Some(Self {
            editor_type,
            language_server,
        })
    }
}

pub async fn create_lsp_client(config: LspConfig, lsp_sender: LspStatusSender) -> LSPClient {
    let indexed = Arc::new(Mutex::new(false));
    let (_, root_path) = config.editor_type.paths().expect("Something went wrong.");

    let (mainloop, mut server) =
        async_lsp::MainLoop::new_client(|_server| {
            let mut router = Router::new(RouterState {
                indexed: indexed.clone(),
                lsp_sender,
                language_server: config.language_server.clone(),
            });
            router
            .notification::<Progress>(|client_state, prog| {
                if matches!(prog.token, NumberOrString::String(s) if s == "rustAnalyzer/Indexing") {
                    match prog.value {
                        ProgressParamsValue::WorkDone(WorkDoneProgress::Begin(begin)) => {
                            *client_state.indexed.lock().unwrap() = false;

                            let mut content =  begin.title;

                            if let Some(message) = begin.message {
                                content.push(' ');
                                content.push_str(&message);
                            }

                            client_state.lsp_sender.send((
                                client_state.language_server.clone(),
                                content
                            )).ok();
                        }
                        ProgressParamsValue::WorkDone(WorkDoneProgress::Report(report)) => {
                            let percentage = report.percentage.map(|v| {
                                if v < 100 {
                                    format!("{v}%")
                                } else {
                                    String::default()
                                }
                            });
                            client_state.lsp_sender.send((
                                client_state.language_server.clone(),
                                format!(
                                    "{} {}",
                                    percentage.unwrap_or_default(),
                                    report.message.clone().unwrap_or_default()
                                ),
                            )).ok();
                        }
                        ProgressParamsValue::WorkDone(WorkDoneProgress::End(end)) => {
                            *client_state.indexed.lock().unwrap() = true;
                            client_state.lsp_sender.send((
                                client_state.language_server.clone(),
                                end.message.unwrap_or_default()
                            )).ok();
                        }
                    }
                }
                ControlFlow::Continue(())
            })
            .notification::<PublishDiagnostics>(|_, _| ControlFlow::Continue(()))
            .notification::<ShowMessage>(|_, _params| ControlFlow::Continue(()))
            .event(|_, _: Stop| ControlFlow::Break(Ok(())));

            ServiceBuilder::new()
                .layer(TracingLayer::default())
                .layer(CatchUnwindLayer::default())
                .layer(ConcurrencyLayer::default())
                .service(router)
        });

    let child = Command::new(config.language_server)
        .current_dir(root_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to start Language Server.");
    let stdout = child.stdout.unwrap();
    let stdin = child.stdin.unwrap();

    let _mainloop_fut = tokio::spawn(async move {
        mainloop.run_bufferred(stdout, stdin).await.ok();
    });

    // Initialize.
    let root_uri = Url::from_file_path(root_path).unwrap();
    let _init_ret = server
        .initialize(InitializeParams {
            root_uri: Some(root_uri),
            capabilities: ClientCapabilities {
                window: Some(WindowClientCapabilities {
                    work_done_progress: Some(true),
                    ..WindowClientCapabilities::default()
                }),
                ..ClientCapabilities::default()
            },
            ..InitializeParams::default()
        })
        .await
        .unwrap();
    server.initialized(InitializedParams {}).unwrap();

    LSPClient {
        indexed,
        server_socket: server,
        language_id: config.editor_type.language_id(),
    }
}

#[derive(Default, Clone, Debug, PartialEq, Copy)]
pub enum LanguageId {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    #[default]
    Unknown,
}

impl Display for LanguageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rust => f.write_str("rust"),
            Self::Python => f.write_str("python"),
            Self::JavaScript => f.write_str("javascript"),
            Self::TypeScript => f.write_str("typescript"),
            Self::Unknown => f.write_str("unknown"),
        }
    }
}

impl LanguageId {
    pub fn parse(id: &str) -> Self {
        match id {
            "rs" => LanguageId::Rust,
            "py" => LanguageId::Python,
            "js" => LanguageId::JavaScript,
            "ts" => LanguageId::TypeScript,
            _ => LanguageId::Unknown,
        }
    }

    pub fn language_server(&self) -> Option<&str> {
        match self {
            LanguageId::Rust => Some("rust-analyzer"),
            _ => None,
        }
    }
}
