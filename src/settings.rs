use std::fs::{read_to_string, write};

use freya::prelude::use_hook;

use crate::state::{AppSettings, RadioAppState};

pub fn load_settings() -> Option<AppSettings> {
    let home_dir = home::home_dir()?;

    let settings_path = home_dir.join("valin.toml");

    // Create if it doesn't exist
    if std::fs::metadata(&settings_path).is_err() {
        let default_settings_content = toml::to_string(&AppSettings::default()).unwrap();
        write(&settings_path, default_settings_content).ok()?;
    }

    let settings_content = read_to_string(&settings_path).ok()?;

    let settings: AppSettings = toml::from_str(&settings_content).ok()?;

    Some(settings)
}

pub fn use_start_watching_settings(radio_app_state: RadioAppState) {
    use_hook(|| {});
}
