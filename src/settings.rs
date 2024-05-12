use std::{
    fs::{read_to_string, write},
    path::PathBuf,
};

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc::channel;
use tracing::info;

use crate::state::{AppSettings, Channel, RadioAppState};

pub fn settings_path() -> Option<PathBuf> {
    let home_dir = home::home_dir()?;

    let settings_path = home_dir.join("valin.toml");

    Some(settings_path)
}

pub fn load_settings() -> Option<AppSettings> {
    let settings_path = settings_path()?;

    // Create if it doesn't exist
    if std::fs::metadata(&settings_path).is_err() {
        let default_settings_content = toml::to_string(&AppSettings::default()).unwrap();
        write(&settings_path, default_settings_content).ok()?;
        info!("Settings file didn't exist, so one was created.");
    }

    let settings_content = read_to_string(&settings_path).ok()?;

    let settings: AppSettings = toml::from_str(&settings_content).ok()?;

    Some(settings)
}

pub async fn watch_settings(mut radio_app_state: RadioAppState) -> Option<()> {
    let (tx, mut rx) = channel::<()>(1);

    let settings_path = settings_path()?;

    let mut watcher = RecommendedWatcher::new(
        move |ev: notify::Result<Event>| {
            if let Ok(ev) = ev {
                if ev.kind.is_modify() {
                    tx.blocking_send(()).unwrap();
                }
            }
        },
        Config::default(),
    )
    .ok()?;

    watcher
        .watch(&settings_path, RecursiveMode::Recursive)
        .ok()?;

    while rx.recv().await.is_some() {
        let settings = load_settings();
        if let Some(settings) = settings {
            let mut app_state = radio_app_state.write_channel(Channel::Settings);
            app_state.set_settings(settings);
        } else {
            info!("Failed to update in-memory settings with the newest changes.")
        }
    }

    Some(())
}
