use gpui::Global;
use serde::{Deserialize, Serialize};

use crate::domain::entities::settings::DbContext;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
    #[default]
    System,
}

impl ThemeMode {
    pub fn next(&self) -> Self {
        match self {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::System,
            ThemeMode::System => ThemeMode::Light,
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            ThemeMode::Light => "sun",
            ThemeMode::Dark => "moon",
            ThemeMode::System => "monitor",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    contexts: Vec<DbContext>,
    #[serde(default)]
    pub theme: ThemeSettings,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ThemeSettings {
    #[serde(default = "default_light_theme")]
    pub light: String,
    #[serde(default = "default_dark_theme")]
    pub dark: String,
    #[serde(default)]
    pub mode: ThemeMode,
}

fn default_light_theme() -> String {
    "Default Light".to_string()
}

fn default_dark_theme() -> String {
    "Default Dark".to_string()
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            light: default_light_theme(),
            dark: default_dark_theme(),
            mode: ThemeMode::default(),
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
