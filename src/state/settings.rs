use serde::{Deserialize, Serialize};
use tracing::info;

use crate::settings::load_settings;

#[derive(Serialize, Deserialize, Debug)]
pub struct EditorSettings {
    pub(crate) font_size: f32,
    pub(crate) line_height: f32,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            font_size: 17.0,
            line_height: 1.2,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AppSettings {
    pub(crate) editor: EditorSettings,
}

impl AppSettings {
    pub fn load() -> Self {
        load_settings().unwrap_or_else(|| {
            info!("Failed to load settings, using defaults.");
            Self::default()
        })
    }
}
