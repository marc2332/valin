use std::{
    fs::{read_to_string, write},
    path::PathBuf,
};

use freya::prelude::use_hook;

use crate::state::{AppSettings, RadioAppState};

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
    }

    let settings_content = read_to_string(&settings_path).ok()?;

    let settings: AppSettings = toml::from_str(&settings_content).ok()?;

    Some(settings)
}
