use std::ops::ControlFlow;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};

use async_lsp::concurrency::ConcurrencyLayer;
use async_lsp::panic::CatchUnwindLayer;
use async_lsp::router::Router;
use async_lsp::tracing::TracingLayer;
use async_lsp::{LanguageServer, ServerSocket};
use async_process::Command;
use dioxus::prelude::Coroutine;
use lsp_types::notification::{Progress, PublishDiagnostics, ShowMessage};
use lsp_types::{
    ClientCapabilities, InitializeParams, InitializedParams, NumberOrString, ProgressParamsValue,
    Url, WindowClientCapabilities, WorkDoneProgress,
};
use tower::ServiceBuilder;

use crate::hooks::EditorManager;

struct ClientState {
    indexed: Arc<Mutex<bool>>,
    lsp_status_coroutine: Coroutine<(String, String)>,
    language_server: String,
}

struct Stop;

#[derive(Clone)]
pub struct LspConfig {
    root_dir: PathBuf,
    pub language_server: String,
}

#[derive(Clone)]
pub struct LSPBridge {
    pub indexed: Arc<Mutex<bool>>,
    pub server_socket: ServerSocket,
}

impl LspConfig {
    pub fn new(root_dir: PathBuf, language_id: LanguageId) -> Option<Self> {
        let language_server = language_id.language_server()?.to_string();

        Some(Self {
            root_dir,
            language_server,
        })
    }
}

pub async fn create_lsp(config: LspConfig, manager: &EditorManager) -> LSPBridge {
    let indexed = Arc::new(Mutex::new(false));

    let (mainloop, mut server) = async_lsp::MainLoop::new_client(|_server| {
        let mut router = Router::new(ClientState {
            indexed: indexed.clone(),
            lsp_status_coroutine: manager.lsp_status_coroutine.clone(),
            language_server: config.language_server.clone(),
        });
        router
            .notification::<Progress>(|this, prog| {
                if matches!(prog.token, NumberOrString::String(s) if s == "rustAnalyzer/Indexing") {
                    if let ProgressParamsValue::WorkDone(WorkDoneProgress::Report(report)) =
                        &prog.value
                    {
                        let percentage = report.percentage.map(|v| {
                            if v < 100 {
                                format!("{v}%")
                            } else {
                                String::default()
                            }
                        });
                        this.lsp_status_coroutine.send((
                            this.language_server.clone(),
                            format!(
                                "{} {}",
                                percentage.unwrap_or_default(),
                                report.message.clone().unwrap_or_default()
                            ),
                        ));
                    }
                    if matches!(
                        prog.value,
                        ProgressParamsValue::WorkDone(WorkDoneProgress::End(_))
                    ) {
                        *this.indexed.lock().unwrap() = true;
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
        .current_dir(&config.root_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed run rust-analyzer");
    let stdout = child.stdout.unwrap();
    let stdin = child.stdin.unwrap();

    let _mainloop_fut = tokio::spawn(async move {
        mainloop.run_bufferred(stdout, stdin).await.unwrap();
    });

    // Initialize.
    let root_uri = Url::from_file_path(&config.root_dir).unwrap();
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

    LSPBridge {
        indexed,
        server_socket: server,
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

impl ToString for LanguageId {
    fn to_string(&self) -> String {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
            Self::Unknown => "unknown",
        }
        .to_string()
    }
}

impl LanguageId {
    pub fn parse(id: &str) -> Self {
        match id {
            "ars" => LanguageId::Rust,
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
