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
use dioxus_std::utils::channel::UseChannel;
use lsp_types::notification::{Progress, PublishDiagnostics, ShowMessage};
use lsp_types::{
    ClientCapabilities, InitializeParams, InitializedParams, NumberOrString, ProgressParamsValue,
    Url, WindowClientCapabilities, WorkDoneProgress,
};
use tower::ServiceBuilder;

use crate::manager::EditorManager;

struct ClientState {
    indexed: Arc<Mutex<bool>>,
    language_servers_status: UseChannel<(String, String)>,
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
    pub fn new(root_dir: PathBuf, language: &str) -> Self {
        #[allow(clippy::match_single_binding)]
        let language_server = match language {
            _ => "rust-analyzer",
        }
        .to_string();

        Self {
            root_dir,
            language_server,
        }
    }
}

pub async fn create_lsp(config: LspConfig, editor_manager: &EditorManager) -> LSPBridge {
    let indexed = Arc::new(Mutex::new(false));
    let language_servers_status = editor_manager.language_servers_status.clone();

    let (mainloop, mut server) = async_lsp::MainLoop::new_client(|_server| {
        let mut router = Router::new(ClientState {
            indexed: indexed.clone(),
            language_servers_status: language_servers_status.clone(),
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
                        this.language_servers_status
                            .try_send((
                                "rust-analyzer".to_string(),
                                format!(
                                    "{} {}",
                                    percentage.unwrap_or_default(),
                                    report.message.clone().unwrap_or_default()
                                ),
                            ))
                            .ok();
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
