pub mod components;
pub mod remindr;
pub mod screens;
pub mod states;

use gpui::{App, SharedString, Window, WindowAppearance};
use gpui_component::theme::{Theme, ThemeRegistry};

use self::states::settings_state::{Settings, ThemeMode};

/// Get theme settings and determine if dark mode should be used
fn get_theme_info(cx: &App) -> (states::settings_state::ThemeSettings, bool) {
    let (theme_settings, mode) = cx
        .try_global::<Settings>()
        .map(|s| (s.theme.clone(), s.theme.mode))
        .unwrap_or_else(|| {
            (
                states::settings_state::ThemeSettings::default(),
                ThemeMode::System,
            )
        });

    let is_system_dark = matches!(
        cx.window_appearance(),
        WindowAppearance::Dark | WindowAppearance::VibrantDark
    );

    let use_dark = match mode {
        ThemeMode::Light => false,
        ThemeMode::Dark => true,
        ThemeMode::System => is_system_dark,
    };

    (theme_settings, use_dark)
}

/// Apply the appropriate theme based on the current settings and system appearance
pub fn apply_theme(_window: &mut Window, cx: &mut App) {
    let (theme_settings, use_dark) = get_theme_info(cx);

    let theme_name: SharedString = if use_dark {
        theme_settings.dark.into()
    } else {
        theme_settings.light.into()
    };

    let theme_config = ThemeRegistry::global(cx).themes().get(&theme_name).cloned();
    if let Some(config) = theme_config {
        Theme::global_mut(cx).apply_config(&config);
        cx.refresh_windows();
    }
}

/// Apply theme globally without requiring a Window reference
/// Used when themes are loaded asynchronously
pub fn apply_theme_global(cx: &mut App) {
    let (theme_settings, use_dark) = get_theme_info(cx);

    let theme_name: SharedString = if use_dark {
        theme_settings.dark.into()
    } else {
        theme_settings.light.into()
    };

    let theme_config = ThemeRegistry::global(cx).themes().get(&theme_name).cloned();
    if let Some(config) = theme_config {
        Theme::global_mut(cx).apply_config(&config);
        cx.refresh_windows();
    }
}
