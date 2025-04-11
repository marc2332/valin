use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::{fmt::Display, ops::ControlFlow};

use async_lsp::concurrency::ConcurrencyLayer;
use async_lsp::panic::CatchUnwindLayer;
use async_lsp::router::Router;
use async_lsp::tracing::TracingLayer;
use async_lsp::{LanguageServer, ServerSocket};
use freya::prelude::spawn_forever;
use lsp_types::{
    notification::{Progress, PublishDiagnostics, ShowMessage},
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, HoverParams, TextDocumentIdentifier,
    TextDocumentItem,
};
use lsp_types::{
    ClientCapabilities, HoverContents, InitializeParams, InitializedParams, MarkedString,
    NumberOrString, Position, ProgressParamsValue, TextDocumentPositionParams, Url,
    WindowClientCapabilities, WorkDoneProgress, WorkDoneProgressParams, WorkspaceFolder,
};
use tokio::process::Command;
use tokio::sync::mpsc::{self, UnboundedSender};
use tower::ServiceBuilder;
use tracing::info;

use crate::state::{AppState, Channel, RadioAppState, TabId};
use crate::views::panels::tabs::editor::{Diagnostics, TabEditorUtils};
use crate::{views::panels::tabs::editor::EditorType, LspStatusSender};

struct RouterState {
    pub(crate) indexed: Arc<Mutex<bool>>,
    pub(crate) lsp_sender: LspStatusSender,
    pub(crate) language_server: String,
}

#[derive(PartialEq, Debug)]
pub struct LspAction {
    pub tab_id: TabId,
    pub action: LspActionData,
}

#[derive(PartialEq, Debug)]
pub enum LspActionData {
    Initialize(PathBuf),
    OpenFile,
    CloseFile { file_uri: Url },
    Hover { position: Position },
    Clear,
}

struct Stop;

#[derive(Clone)]
pub struct LSPClient {
    pub(crate) tx: UnboundedSender<LspAction>,
}

impl LSPClient {
    pub fn send(&self, action: LspAction) {
        self.tx.send(action).unwrap();
    }

    pub fn open_with(
        mut radio_app_state: RadioAppState,
        app_state: &mut AppState,
        lsp_config: &LspConfig,
    ) -> (Self, bool) {
        let server = app_state.lsp(lsp_config).cloned();
        match server {
            Some(server) => (server, false),
            None => {
                let lsp_sender = app_state.lsp_sender.clone();
                let (tx, mut rx) = mpsc::unbounded_channel::<LspAction>();
                let (client, indexed, mut server) = Self::new(lsp_config, lsp_sender, tx);
                let language_id = lsp_config.editor_type.language_id();
                spawn_forever({
                    async move {
                        while let Some(action) = rx.recv().await {
                            let is_indexed = *indexed.lock().unwrap();
                            match action.action {
                                LspActionData::Initialize(root_path) if !is_indexed => {
                                    let root_uri = Url::from_file_path(&root_path).unwrap();
                                    let _init_ret = server
                                        .initialize(InitializeParams {
                                            workspace_folders: Some(vec![WorkspaceFolder {
                                                uri: root_uri,
                                                name: root_path.display().to_string(),
                                            }]),
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
                                }
                                LspActionData::OpenFile => {
                                    let app_state = radio_app_state.read();
                                    let tab = app_state.tab(&action.tab_id);
                                    let editor_tab = tab.as_text_editor().unwrap();
                                    let Some(file_uri) = editor_tab.editor.uri() else {
                                        return;
                                    };
                                    let file_content = editor_tab.editor.content();
                                    info!("Opened document [uri={file_uri}]",);
                                    server
                                        .did_open(DidOpenTextDocumentParams {
                                            text_document: TextDocumentItem {
                                                uri: file_uri,
                                                language_id: language_id.to_string(),
                                                version: 0,
                                                text: file_content,
                                            },
                                        })
                                        .unwrap();
                                }

                                LspActionData::CloseFile { file_uri } => {
                                    info!("Closed document [uri={file_uri}] from LSP");
                                    server
                                        .did_close(DidCloseTextDocumentParams {
                                            text_document: TextDocumentIdentifier { uri: file_uri },
                                        })
                                        .unwrap();
                                }
                                LspActionData::Hover { position } if is_indexed => {
                                    let Some(file_uri) = ({
                                        let app_state = radio_app_state.read();
                                        let tab = app_state.tab(&action.tab_id);
                                        let editor_tab = tab.as_text_editor().unwrap();
                                        editor_tab.editor.uri()
                                    }) else {
                                        return;
                                    };
                                    let response = server
                                        .hover(HoverParams {
                                            text_document_position_params:
                                                TextDocumentPositionParams {
                                                    text_document: TextDocumentIdentifier {
                                                        uri: file_uri,
                                                    },
                                                    position,
                                                },
                                            work_done_progress_params:
                                                WorkDoneProgressParams::default(),
                                        })
                                        .await;
                                    if let Ok(Some(response)) = response {
                                        let content = match response.contents {
                                            HoverContents::Markup(contents) => {
                                                contents.value.to_owned()
                                            }
                                            HoverContents::Array(contents) => contents
                                                .iter()
                                                .map(|v| match v {
                                                    MarkedString::String(v) => v.to_owned(),
                                                    MarkedString::LanguageString(text) => {
                                                        text.value.to_owned()
                                                    }
                                                })
                                                .collect::<Vec<String>>()
                                                .join("\n"),
                                            HoverContents::Scalar(v) => match v {
                                                MarkedString::String(v) => v.to_owned(),
                                                MarkedString::LanguageString(text) => {
                                                    text.value.to_owned()
                                                }
                                            },
                                        };
                                        if content != "()" {
                                            let mut app_state = radio_app_state
                                                .write_channel(Channel::follow_tab(action.tab_id));
                                            let tab = app_state.tab_mut(&action.tab_id);
                                            let editor_tab = tab.as_text_editor_mut().unwrap();
                                            editor_tab.editor.diagnostics = Some(Diagnostics {
                                                range: response.range.unwrap_or_default(),
                                                content,
                                                line: position.line,
                                            })
                                        }
                                    }
                                }
                                LspActionData::Clear => {
                                    let mut app_state = radio_app_state
                                        .write_channel(Channel::follow_tab(action.tab_id));
                                    let tab = app_state.tab_mut(&action.tab_id);
                                    let editor_tab = tab.as_text_editor_mut().unwrap();
                                    editor_tab.editor.diagnostics.take();
                                }
                                _ if is_indexed => {
                                    info!("Language Server is indexing.");
                                }
                                _ => {}
                            }
                        }
                    }
                });
                (client, true)
            }
        }
    }

    pub fn new(
        config: &LspConfig,
        lsp_sender: LspStatusSender,
        tx: UnboundedSender<LspAction>,
    ) -> (Self, Arc<Mutex<bool>>, ServerSocket) {
        let indexed = Arc::new(Mutex::new(false));
        let (_, root_path) = config.editor_type.paths().expect("Something went wrong.");

        let (mainloop, server) = async_lsp::MainLoop::new_client(|_server| {
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

        let child = Command::new(&config.language_server)
            .current_dir(root_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to start Language Server.");
        let stdout = tokio_util::compat::TokioAsyncReadCompatExt::compat(child.stdout.unwrap());
        let stdin =
            tokio_util::compat::TokioAsyncWriteCompatExt::compat_write(child.stdin.unwrap());

        let _mainloop_fut = spawn_forever(async move {
            mainloop.run_buffered(stdout, stdin).await.ok();
        });

        (LSPClient { tx }, indexed, server)
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

#[derive(Default, Clone, Debug, PartialEq, Copy)]
pub enum LanguageId {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Markdown,
    #[default]
    Unknown,
}

impl Display for LanguageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rust => f.write_str("Rust"),
            Self::Python => f.write_str("Python"),
            Self::JavaScript => f.write_str("JavaScript"),
            Self::TypeScript => f.write_str("TypeScript"),
            Self::Markdown => f.write_str("Markdown"),
            Self::Unknown => f.write_str("Unknown"),
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
            "md" => LanguageId::Markdown,
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
