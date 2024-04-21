#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod commands;
mod components;
mod constants;
mod hooks;
mod lsp;
mod parser;
mod state;
mod tabs;
mod utils;

use std::sync::Arc;

use crate::app::App;
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
}

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive("freya_editor=debug".parse().unwrap())
                .from_env()
                .unwrap(),
        )
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args = Args::parse();

    info!("Starting freya-editor. \n{args:#?}");

    launch_cfg(
        || {
            rsx!(
                ThemeProvider { theme: CUSTOM_THEME, App {} }
            )
        },
        LaunchConfig::<Arc<Args>>::builder()
            .with_width(900.0)
            .with_height(600.0)
            .with_title("freya-editor")
            .with_state(Arc::new(args))
            .build(),
    );
}
