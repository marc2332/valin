#[allow(non_snake_case)]
pub mod Settings {
    use std::path::Path;

    use async_trait::async_trait;

    use crate::{
        fs::FSReadTransportInterface,
        settings::settings_path,
        state::{AppState, RadioAppState},
        views::panels::tabs::editor::EditorTab,
    };

    pub fn open_with(radio_app_state: RadioAppState, app_state: &mut AppState) {
        let settings_path = settings_path().unwrap();

        EditorTab::open_with(
            radio_app_state,
            app_state,
            settings_path.clone(),
            settings_path,
            Box::new(MemoryTransport(
                toml::to_string(&app_state.settings).unwrap(),
            )),
        );
    }

    struct MemoryTransport(String);

    #[async_trait]
    impl FSReadTransportInterface for MemoryTransport {
        async fn read_to_string(&self, _path: &Path) -> tokio::io::Result<String> {
            Ok(self.0.clone())
        }
    }
}
