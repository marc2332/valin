use serde::{Deserialize, Serialize, Serializer};
use tracing::info;

use crate::settings::load_settings;

fn human_number_serializer<S>(value: &f32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_f64((*value as f64 * 100.0).trunc() / 100.0)
}
#[derive(Serialize, Deserialize, Debug)]
pub struct EditorSettings {
    #[serde(serialize_with = "human_number_serializer")]
    pub(crate) font_size: f32,
    #[serde(serialize_with = "human_number_serializer")]
    pub(crate) line_height: f32,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            font_size: 17.0,
            line_height: 1.6_f32,
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
