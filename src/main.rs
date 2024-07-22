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
mod keyboard_navigation;
mod lsp;
mod metrics;
mod parser;
mod settings;
mod state;
mod tabs;
mod utils;

use std::{path::PathBuf, sync::Arc};

use crate::app::App;
use crate::keyboard_navigation::KeyboardNavigationProvider;
use clap::Parser;
use components::*;
use freya::prelude::*;
use hooks::*;
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

const CUSTOM_THEME: Theme = Theme {
    button: ButtonTheme {
        border_fill: Cow::Borrowed("rgb(50, 50, 50)"),
        ..DARK_THEME.button
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

    launch_cfg(
        || {
            rsx!(
                ThemeProvider {
                    theme: CUSTOM_THEME,
                    KeyboardNavigationProvider {
                        App {}
                    }
                }
            )
        },
        LaunchConfig::<Arc<Args>>::default()
            .with_size(1280.0, 720.0)
            .with_title("Valin")
            .with_state(Arc::new(args)),
    );
}
