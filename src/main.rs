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
mod theme;
mod views;

use std::path::PathBuf;

use crate::app::AppView;
use clap::Parser;
use freya::prelude::*;
use freya_performance_plugin::PerformanceOverlayPlugin;
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

/// Fall back to the Adwaita cursor theme when the host theme isn't reachable
/// inside the Flatpak sandbox (causes an invisible cursor on Wayland).
#[cfg(target_os = "linux")]
fn fix_flatpak_cursor_theme() {
    if std::env::var("FLATPAK_ID").is_err() {
        return;
    }

    let theme_name = std::env::var("XCURSOR_THEME").unwrap_or_else(|_| "default".into());

    if xcursor::CursorTheme::load(&theme_name)
        .load_icon("left_ptr")
        .is_none()
    {
        // SAFETY: called before any other threads are spawned.
        unsafe {
            std::env::set_var("XCURSOR_THEME", "Adwaita");
        }
    }

    if std::env::var("XCURSOR_SIZE").is_err() {
        // SAFETY: called before any other threads are spawned.
        unsafe {
            std::env::set_var("XCURSOR_SIZE", "24");
        }
    }
}

#[derive(Parser, Debug, PartialEq, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    // Open certain folders or files.
    #[arg(num_args(0..))]
    paths: Vec<PathBuf>,

    /// Enable the FPS overlay.
    #[arg(long)]
    fps: bool,
}

fn main() {
    #[cfg(target_os = "linux")]
    fix_flatpak_cursor_theme();

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

    if args.fps {
        config = config.with_plugin(PerformanceOverlayPlugin::default())
    }

    launch(
        config.with_window(
            WindowConfig::new_app(AppView(args.clone()))
                .with_size(1100.0, 800.0)
                .with_title("Valin"),
        ),
    );
}
