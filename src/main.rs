#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod components;
mod fs;
mod global_defaults;
mod settings;
mod state;
mod views;

use std::path::PathBuf;

use crate::app::AppView;
use clap::Parser;
use freya::prelude::*;
use freya_performance_plugin::PerformanceOverlayPlugin;
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[derive(Parser, Debug, PartialEq, Clone)]
#[command(version, about, long_about = None)]
struct Args {
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
                .unwrap()
                .add_directive("freya::radio=debug".parse().unwrap()),
        )
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args = Args::parse();

    info!("Starting valin. \n{args:#?}");

    let mut config = LaunchConfig::default();

    if args.performance_overlay {
        config = config.with_plugin(PerformanceOverlayPlugin::default())
    }

    launch(
        config.with_window(
            WindowConfig::new_app(AppView(args.clone()))
                .with_size(1280.0, 720.0)
                .with_title("Valin"),
        ),
    );
}
