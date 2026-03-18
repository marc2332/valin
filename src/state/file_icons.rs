use std::{collections::HashMap, path::Path};

use freya::prelude::Bytes;
use include_dir::{Dir, include_dir};
use serde::Deserialize;

static MATERIAL_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/icons/themes/material");

#[derive(Deserialize)]
struct ThemeJson {
    file_default: String,
    folder_open: String,
    folder_closed: String,
    icons: Vec<IconJson>,
}

#[derive(Deserialize)]
struct IconJson {
    svg: String,
    extensions: Vec<String>,
}

/// A resolved icon: owned SVG bytes + display name.
#[derive(Clone)]
pub struct IconEntry {
    pub svg: Bytes,
}

/// A loaded icon theme, built from a `theme.json` and the SVG files alongside it.
pub struct IconTheme {
    extension_map: HashMap<String, IconEntry>,
    file_default: IconEntry,
    folder_open: IconEntry,
    folder_closed: IconEntry,
}

impl IconTheme {
    /// Load a theme from a `theme.json` string and a directory that contains
    /// the SVG files referenced by that JSON.
    pub fn from_json(json: &str, dir: &Dir<'_>) -> Result<Self, serde_json::Error> {
        let theme: ThemeJson = serde_json::from_str(json)?;

        let resolve = |filename: &str| -> IconEntry {
            let svg = dir
                .get_file(filename)
                .unwrap_or_else(|| panic!("icon theme references missing file: {filename}"))
                .contents();
            IconEntry {
                svg: Bytes::copy_from_slice(svg),
            }
        };

        let file_default = resolve(&theme.file_default);
        let folder_open = resolve(&theme.folder_open);
        let folder_closed = resolve(&theme.folder_closed);

        let mut extension_map = HashMap::new();
        for icon in &theme.icons {
            let entry = IconEntry {
                svg: Bytes::copy_from_slice(
                    dir.get_file(&icon.svg)
                        .unwrap_or_else(|| {
                            panic!("icon theme references missing file: {}", icon.svg)
                        })
                        .contents(),
                ),
            };
            for ext in &icon.extensions {
                extension_map.insert(ext.clone(), entry.clone());
            }
        }

        Ok(Self {
            extension_map,
            file_default,
            folder_open,
            folder_closed,
        })
    }

    /// Resolves the icon for a file path by extension.
    /// Falls back to the theme's `file_default` for unknown or missing extensions.
    pub fn get_file(&self, path: &Path) -> &IconEntry {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(|e| self.extension_map.get(&e.to_lowercase()))
            .unwrap_or(&self.file_default)
    }

    /// Resolves the folder icon (open or closed).
    pub fn get_folder(&self, open: bool) -> &IconEntry {
        if open {
            &self.folder_open
        } else {
            &self.folder_closed
        }
    }
}

/// Holds all available icon themes. One theme is active at a time.
/// Initialized with the built-in Material Icons theme.
pub struct FileIcons {
    themes: Vec<IconTheme>,
    active: usize,
}

impl FileIcons {
    pub fn new() -> Self {
        let json = MATERIAL_DIR
            .get_file("theme.json")
            .expect("built-in material theme.json is missing")
            .contents_utf8()
            .expect("theme.json must be valid UTF-8");

        let material = IconTheme::from_json(json, &MATERIAL_DIR)
            .expect("built-in material theme must be valid JSON");

        Self {
            themes: vec![material],
            active: 0,
        }
    }

    pub fn active_theme(&self) -> &IconTheme {
        &self.themes[self.active]
    }

    pub fn get_file(&self, path: &Path) -> &IconEntry {
        self.active_theme().get_file(path)
    }

    pub fn get_folder(&self, open: bool) -> &IconEntry {
        self.active_theme().get_folder(open)
    }
}

impl Default for FileIcons {
    fn default() -> Self {
        Self::new()
    }
}
