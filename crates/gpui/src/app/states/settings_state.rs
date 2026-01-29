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

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    contexts: Vec<DbContext>,
    #[serde(default)]
    pub theme: ThemeSettings,
    #[serde(default)]
    pub appearance: AppearanceSettings,
    #[serde(default)]
    pub editor: EditorSettings,
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

#[derive(Serialize, Deserialize, Clone)]
pub struct AppearanceSettings {
    #[serde(default = "default_ui_font_size")]
    pub ui_font_size: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EditorSettings {
    #[serde(default = "default_editor_font_size")]
    pub font_size: f32,
    #[serde(default = "default_zoom")]
    pub zoom: f32,
    #[serde(default)]
    pub disabled_blocks: Vec<String>,
    #[serde(default)]
    pub block_font_sizes: BlockFontSizes,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BlockFontSizes {
    #[serde(default = "default_h1_font_size")]
    pub heading_1: f32,
    #[serde(default = "default_h2_font_size")]
    pub heading_2: f32,
    #[serde(default = "default_h3_font_size")]
    pub heading_3: f32,
    #[serde(default = "default_text_font_size")]
    pub text: f32,
}

fn default_light_theme() -> String {
    "Default Light".to_string()
}

fn default_dark_theme() -> String {
    "Default Dark".to_string()
}

fn default_ui_font_size() -> f32 {
    14.0
}

fn default_editor_font_size() -> f32 {
    16.0
}

fn default_zoom() -> f32 {
    1.0
}

fn default_h1_font_size() -> f32 {
    30.0
}

fn default_h2_font_size() -> f32 {
    24.0
}

fn default_h3_font_size() -> f32 {
    20.0
}

fn default_text_font_size() -> f32 {
    16.0
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

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            ui_font_size: default_ui_font_size(),
        }
    }
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            font_size: default_editor_font_size(),
            zoom: default_zoom(),
            disabled_blocks: Vec::new(),
            block_font_sizes: BlockFontSizes::default(),
        }
    }
}

impl Default for BlockFontSizes {
    fn default() -> Self {
        Self {
            heading_1: default_h1_font_size(),
            heading_2: default_h2_font_size(),
            heading_3: default_h3_font_size(),
            text: default_text_font_size(),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            contexts: Vec::new(),
            theme: ThemeSettings::default(),
            appearance: AppearanceSettings::default(),
            editor: EditorSettings::default(),
        }
    }
}

impl Settings {
    pub fn save(&self) {
        if let Some(home) = dirs::home_dir() {
            let config_path = if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
                home.join(".config").join("remindr")
            } else {
                dirs::config_dir()
                    .unwrap_or(home.join(".config"))
                    .join("remindr")
            };

            let settings_file = config_path.join("settings.json");
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(settings_file, json);
            }
        }
    }
}

impl Global for Settings {}
