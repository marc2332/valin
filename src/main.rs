#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod components;
mod constants;
mod fs;
mod global_defaults;
mod hooks;
mod lsp;
mod metrics;
mod parser;
mod settings;
mod state;
mod utils;
mod views;

use std::{path::PathBuf, sync::Arc};

use crate::app::App;
use clap::Parser;
use components::*;
use freya::prelude::*;
use hooks::*;
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

const CUSTOM_THEME: Theme = Theme {
    button: ButtonTheme {
        border_fill: Cow::Borrowed("rgb(45, 49, 50)"),
        background: Cow::Borrowed("rgb(28, 31, 32)"),
        ..DARK_THEME.button
    },
    input: InputTheme {
        background: Cow::Borrowed("rgb(28, 31, 32)"),
        ..DARK_THEME.input
    },
    ..DARK_THEME
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Enable Support for language servers.
    #[arg(short, long)]
    lsp: bool,

    // Open certain folders or files.
    #[arg(num_args(0..))]
    paths: Vec<PathBuf>,

    /// Enable the performance overlay.
    #[arg(short, long)]
    performance_overlay: bool,
}

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive("valin=debug".parse().unwrap())
                .from_env()
                .unwrap(),
        )
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args = Args::parse();

    info!("Starting valin. \n{args:#?}");

    let mut config = LaunchConfig::<Arc<Args>>::default();

    if args.performance_overlay {
        config = config.with_plugin(PerformanceOverlayPlugin::default())
    }

    launch_cfg(
        || {
            rsx!(
                ThemeProvider {
                    theme: CUSTOM_THEME,
                    App {}
                }
            )
        },
        config
            .with_size(1280.0, 720.0)
            .with_title("Valin")
            .with_state(Arc::new(args)), // .with_max_paragraph_cache_size(200),
    );
}
