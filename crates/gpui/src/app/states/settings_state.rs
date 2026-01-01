use gpui::Global;
use serde::{Deserialize, Serialize};

use crate::domain::entities::settings::DbContext;

#[derive(Serialize, Deserialize)]
pub struct Settings {
    contexts: Vec<DbContext>,
    #[serde(default)]
    pub theme: ThemeSettings,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ThemeSettings {
    #[serde(default = "default_theme_name")]
    pub name: String,
}

fn default_theme_name() -> String {
    "Default Dark".to_string()
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            name: default_theme_name(),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            contexts: Vec::new(),
            theme: ThemeSettings::default(),
        }
    }
}

impl Global for Settings {}
