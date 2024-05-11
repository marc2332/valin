#[allow(non_snake_case)]
pub mod Settings {
    use crate::{settings::settings_path, state::AppState, tabs::editor::EditorTab};

    pub fn open_with(app_state: &mut AppState) {
        let settings_path = settings_path().unwrap();
        EditorTab::open_with(
            app_state,
            settings_path.clone(),
            settings_path,
            toml::to_string(&app_state.settings).unwrap(),
        );
    }
}
