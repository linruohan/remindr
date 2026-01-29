use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, Icon, IconName, Root, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState, NumberInput, NumberInputEvent},
    label::Label,
    popover::Popover,
    scroll::ScrollableElement,
    switch::Switch,
    theme::ThemeRegistry,
    v_flex,
};

use crate::app::{
    apply_theme,
    states::settings_state::{Settings, ThemeMode},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsSection {
    Appearance,
    Editor,
}

struct BlockType {
    id: &'static str,
    label: &'static str,
    icon_path: &'static str,
}

const BLOCK_TYPES: &[BlockType] = &[
    BlockType {
        id: "text",
        label: "Text",
        icon_path: "icons/pilcrow.svg",
    },
    BlockType {
        id: "heading_2",
        label: "Heading 2",
        icon_path: "icons/heading-2.svg",
    },
    BlockType {
        id: "heading_3",
        label: "Heading 3",
        icon_path: "icons/heading-3.svg",
    },
    BlockType {
        id: "divider",
        label: "Divider",
        icon_path: "icons/separator-horizontal.svg",
    },
];

pub struct SettingsWindow {
    active_section: SettingsSection,
    ui_font_size_input: Entity<InputState>,
    editor_font_size_input: Entity<InputState>,
    zoom_input: Entity<InputState>,
    light_theme_search: Entity<InputState>,
    dark_theme_search: Entity<InputState>,
}

impl SettingsWindow {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let settings = cx.global::<Settings>().clone();

        let ui_font_size_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            state.set_value(format!("{}", settings.appearance.ui_font_size), window, cx);
            state
        });

        let editor_font_size_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            state.set_value(format!("{}", settings.editor.font_size), window, cx);
            state
        });

        let zoom_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            state.set_value(format!("{}", settings.editor.zoom), window, cx);
            state
        });

        cx.subscribe_in(
            &ui_font_size_input,
            window,
            |this, _, event: &InputEvent, _, cx| {
                if let InputEvent::Change = event {
                    this.on_ui_font_size_changed(cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &editor_font_size_input,
            window,
            |this, _, event: &InputEvent, _, cx| {
                if let InputEvent::Change = event {
                    this.on_editor_font_size_changed(cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(&zoom_input, window, |this, _, event: &InputEvent, _, cx| {
            if let InputEvent::Change = event {
                this.on_zoom_changed(cx);
            }
        })
        .detach();

        cx.subscribe_in(
            &ui_font_size_input,
            window,
            |this, _, _: &NumberInputEvent, _, cx| {
                this.on_ui_font_size_changed(cx);
            },
        )
        .detach();

        cx.subscribe_in(
            &editor_font_size_input,
            window,
            |this, _, _: &NumberInputEvent, _, cx| {
                this.on_editor_font_size_changed(cx);
            },
        )
        .detach();

        cx.subscribe_in(
            &zoom_input,
            window,
            |this, _, _: &NumberInputEvent, _, cx| {
                this.on_zoom_changed(cx);
            },
        )
        .detach();

        let light_theme_search = cx.new(|cx| InputState::new(window, cx).placeholder("Search..."));
        let dark_theme_search = cx.new(|cx| InputState::new(window, cx).placeholder("Search..."));

        cx.subscribe_in(
            &light_theme_search,
            window,
            |_this, _, event: &InputEvent, _, cx| {
                if let InputEvent::Change = event {
                    cx.notify();
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &dark_theme_search,
            window,
            |_this, _, event: &InputEvent, _, cx| {
                if let InputEvent::Change = event {
                    cx.notify();
                }
            },
        )
        .detach();

        // Re-render when global settings change (e.g. from file watcher)
        cx.observe_global::<Settings>(|_this, cx| {
            cx.notify();
        })
        .detach();

        Self {
            active_section: SettingsSection::Appearance,
            ui_font_size_input,
            editor_font_size_input,
            zoom_input,
            light_theme_search,
            dark_theme_search,
        }
    }

    fn on_ui_font_size_changed(&self, cx: &mut Context<Self>) {
        let value = self.ui_font_size_input.read(cx).value();
        if let Ok(size) = value.parse::<f32>() {
            let size = size.clamp(10.0, 24.0);
            cx.update_global::<Settings, _>(|settings, _| {
                settings.appearance.ui_font_size = size;
                settings.save();
            });
        }
    }

    fn on_editor_font_size_changed(&self, cx: &mut Context<Self>) {
        let value = self.editor_font_size_input.read(cx).value();
        if let Ok(size) = value.parse::<f32>() {
            let size = size.clamp(10.0, 32.0);
            cx.update_global::<Settings, _>(|settings, _| {
                settings.editor.font_size = size;
                settings.save();
            });
        }
    }

    fn on_zoom_changed(&self, cx: &mut Context<Self>) {
        let value = self.zoom_input.read(cx).value();
        if let Ok(zoom) = value.parse::<f32>() {
            let zoom = zoom.clamp(0.5, 2.0);
            cx.update_global::<Settings, _>(|settings, _| {
                settings.editor.zoom = zoom;
                settings.save();
            });
        }
    }

    pub fn open(cx: &mut App) {
        let window_size = size(px(860.), px(600.));
        let window_bounds = Bounds::centered(None, window_size, cx);

        cx.spawn(async move |cx| {
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(window_bounds)),
                window_min_size: Some(Size {
                    width: px(640.),
                    height: px(400.),
                }),
                kind: WindowKind::Normal,
                titlebar: Some(TitlebarOptions {
                    appears_transparent: true,
                    title: Some("Settings".into()),
                    traffic_light_position: Some(point(px(9.0), px(9.0))),
                    ..Default::default()
                }),
                ..Default::default()
            };

            let window = cx
                .open_window(options, |window, cx| {
                    let view = cx.new(|cx| SettingsWindow::new(window, cx));
                    cx.new(|cx| Root::new(view, window, cx))
                })
                .expect("failed to open settings window");

            window
                .update(cx, |_, window, _| {
                    window.activate_window();
                    window.set_window_title("Settings");
                    window.set_background_appearance(gpui::WindowBackgroundAppearance::Opaque);
                })
                .expect("failed to update settings window");

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    }

    fn open_settings_json() {
        if let Some(home) = dirs::home_dir() {
            let config_path = if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
                home.join(".config").join("remindr")
            } else {
                dirs::config_dir()
                    .unwrap_or(home.join(".config"))
                    .join("remindr")
            };
            let settings_file = config_path.join("settings.json");
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open")
                    .arg(&settings_file)
                    .spawn();
            }
            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("xdg-open")
                    .arg(&settings_file)
                    .spawn();
            }
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("cmd")
                    .args(["/C", "start", "", &settings_file.to_string_lossy()])
                    .spawn();
            }
        }
    }

    // -- Rendering helpers --

    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let sections = [
            (
                SettingsSection::Appearance,
                "Appearance",
                "icons/palette.svg",
            ),
            (SettingsSection::Editor, "Editor", "icons/file-text.svg"),
        ];

        let active = self.active_section;
        let accent_bg = cx.theme().sidebar_accent;
        let fg = cx.theme().foreground;
        let muted_fg = cx.theme().muted_foreground;

        v_flex()
            .w(px(200.0))
            .h_full()
            .flex_shrink_0()
            .border_r_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().sidebar)
            .pt_8()
            .px_2()
            .gap_0p5()
            .children(
                sections
                    .into_iter()
                    .map(move |(section, label, icon_path)| {
                        let is_active = active == section;
                        div()
                            .id(SharedString::from(format!("section-{}", label)))
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_2()
                            .py_1p5()
                            .rounded_md()
                            .cursor_pointer()
                            .text_sm()
                            .when(is_active, |el| el.bg(accent_bg).text_color(fg))
                            .when(!is_active, |el| {
                                el.text_color(muted_fg)
                                    .hover(|el| el.bg(accent_bg.opacity(0.5)))
                            })
                            .child(
                                gpui_component::Icon::default()
                                    .path(icon_path)
                                    .size_4()
                                    .text_color(if is_active { fg } else { muted_fg }),
                            )
                            .child(label)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.active_section = section;
                                cx.notify();
                            }))
                    }),
            )
    }

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .w_full()
            .justify_between()
            .items_center()
            .pb_4()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .px_2()
                            .py_0p5()
                            .rounded_sm()
                            .bg(cx.theme().accent)
                            .text_xs()
                            .text_color(cx.theme().accent_foreground)
                            .child("User"),
                    )
                    .child(
                        Label::new("remindr")
                            .text_sm()
                            .text_color(cx.theme().foreground),
                    ),
            )
            .child(
                Button::new("edit-settings-json")
                    .small()
                    .ghost()
                    .label("Edit in settings.json")
                    .on_click(|_, _, _| {
                        Self::open_settings_json();
                    }),
            )
    }

    fn render_section_title(title: &str, fg: Hsla) -> AnyElement {
        div()
            .text_lg()
            .font_semibold()
            .text_color(fg)
            .pb_2()
            .child(title.to_string())
            .into_any_element()
    }

    fn render_sub_header(label: &str, fg: Hsla) -> AnyElement {
        Label::new(label.to_string())
            .text_sm()
            .font_semibold()
            .text_color(fg)
            .into_any_element()
    }

    fn render_setting_row(
        label: &str,
        description: &str,
        control: AnyElement,
        fg: Hsla,
        muted_fg: Hsla,
        border: Hsla,
    ) -> AnyElement {
        h_flex()
            .w_full()
            .justify_between()
            .items_center()
            .py_3()
            .border_b_1()
            .border_color(border)
            .child(
                v_flex()
                    .flex_1()
                    .gap_0p5()
                    .child(
                        Label::new(label.to_string())
                            .text_sm()
                            .font_semibold()
                            .text_color(fg),
                    )
                    .child(
                        Label::new(description.to_string())
                            .text_xs()
                            .text_color(muted_fg),
                    ),
            )
            .child(div().flex_shrink_0().ml_4().child(control))
            .into_any_element()
    }

    fn render_appearance_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = cx.global::<Settings>().clone();
        let current_mode = settings.theme.mode;

        let fg = cx.theme().foreground;
        let muted_fg = cx.theme().muted_foreground;
        let secondary_fg = cx.theme().secondary_foreground;
        let border = cx.theme().border;

        let theme_mode_control = self
            .render_theme_mode_toggle(current_mode, cx)
            .into_any_element();
        let light_theme_control = self
            .render_theme_dropdown(
                "light-theme",
                &settings.theme.light,
                false,
                &self.light_theme_search.clone(),
                cx,
            )
            .into_any_element();
        let dark_theme_control = self
            .render_theme_dropdown(
                "dark-theme",
                &settings.theme.dark,
                true,
                &self.dark_theme_search.clone(),
                cx,
            )
            .into_any_element();
        let ui_font_control = NumberInput::new(&self.ui_font_size_input)
            .small()
            .into_any_element();

        v_flex()
            .gap_1()
            .child(Self::render_section_title("Appearance", fg))
            .child(Self::render_sub_header("Theme", secondary_fg))
            .child(Self::render_setting_row(
                "Theme Mode",
                "Choose between light, dark, or system theme.",
                theme_mode_control,
                fg,
                muted_fg,
                border,
            ))
            .child(Self::render_setting_row(
                "Light Theme",
                "Theme used in light mode.",
                light_theme_control,
                fg,
                muted_fg,
                border,
            ))
            .child(Self::render_setting_row(
                "Dark Theme",
                "Theme used in dark mode.",
                dark_theme_control,
                fg,
                muted_fg,
                border,
            ))
            .child(
                div()
                    .pt_4()
                    .child(Self::render_sub_header("Font", secondary_fg)),
            )
            .child(Self::render_setting_row(
                "UI Font Size",
                "Font size for the application interface (10-24).",
                ui_font_control,
                fg,
                muted_fg,
                border,
            ))
    }

    fn render_theme_mode_toggle(
        &self,
        current: ThemeMode,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let (icon_path, tooltip) = match current {
            ThemeMode::Light => ("icons/sun.svg", "Light — click to switch to Dark"),
            ThemeMode::Dark => ("icons/moon.svg", "Dark — click to switch to System"),
            ThemeMode::System => ("icons/sun-moon.svg", "System — click to switch to Light"),
        };

        let next_mode = match current {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::System,
            ThemeMode::System => ThemeMode::Light,
        };

        Button::new("theme-mode-toggle")
            .small()
            .ghost()
            .icon(gpui_component::Icon::default().path(icon_path))
            .tooltip(tooltip)
            .on_click(cx.listener(move |_, _, window, cx| {
                cx.update_global::<Settings, _>(|settings, _| {
                    settings.theme.mode = next_mode;
                    settings.save();
                });
                apply_theme(window, cx);
                cx.notify();
            }))
    }

    fn render_theme_dropdown(
        &self,
        id: &str,
        current: &str,
        is_dark: bool,
        search_input: &Entity<InputState>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let current = current.to_string();
        let theme_registry = ThemeRegistry::global(cx);
        let all_themes = theme_registry.sorted_themes();

        let themes: Vec<String> = all_themes
            .iter()
            .filter(|t| t.mode.is_dark() == is_dark)
            .map(|t| t.name.to_string())
            .collect();

        let search_query = search_input.read(cx).value().to_lowercase();
        let filtered_themes: Vec<String> = themes
            .into_iter()
            .filter(|name| search_query.is_empty() || name.to_lowercase().contains(&search_query))
            .collect();

        let search_input = search_input.clone();
        let search_input_for_close = search_input.clone();
        let bg = cx.theme().background;
        let border = cx.theme().border;
        let fg = cx.theme().foreground;
        let accent = cx.theme().accent;
        let accent_fg = cx.theme().accent_foreground;
        let muted_fg = cx.theme().muted_foreground;
        let hover_bg = cx.theme().secondary;

        Popover::new(SharedString::from(format!("{}-popover", id)))
            .anchor(Corner::TopRight)
            .trigger(ThemeDropdownTrigger {
                id: SharedString::from(id.to_string()).into(),
                label: current.clone(),
                muted_fg,
                fg,
                input_border: border,
                ring: cx.theme().ring,
                bg,
                radius: cx.theme().radius,
                selected: false,
            })
            .appearance(false)
            .on_open_change(move |open, window, cx| {
                if !open {
                    search_input_for_close.update(cx, |state, cx| {
                        state.set_value("", window, cx);
                    });
                }
            })
            .content(move |_, _, _| {
                v_flex()
                    .w(px(200.))
                    .mt_1()
                    .bg(bg)
                    .border_1()
                    .border_color(border)
                    .rounded_md()
                    .shadow_md()
                    .overflow_hidden()
                    .child(
                        div().p_1().border_b_1().border_color(border).child(
                            Input::new(&search_input)
                                .small()
                                .appearance(false)
                                .prefix(Icon::new(IconName::Search).xsmall().text_color(muted_fg)),
                        ),
                    )
                    .child(
                        v_flex()
                            .max_h(px(200.))
                            .overflow_y_scrollbar()
                            .p_1()
                            .when(filtered_themes.is_empty(), |el| {
                                el.child(
                                    v_flex()
                                        .items_center()
                                        .gap_1()
                                        .py_4()
                                        .child(
                                            Icon::new(IconName::Inbox)
                                                .size_5()
                                                .text_color(muted_fg),
                                        )
                                        .child(
                                            Label::new("No themes found")
                                                .text_xs()
                                                .text_color(muted_fg),
                                        ),
                                )
                            })
                            .children(filtered_themes.iter().map(|theme_name| {
                                let is_selected = current == *theme_name;
                                let name_for_click = theme_name.clone();

                                div()
                                    .id(SharedString::from(format!("theme-{}", theme_name)))
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .px_2()
                                    .py_1()
                                    .rounded_sm()
                                    .cursor_pointer()
                                    .text_sm()
                                    .when(is_selected, |el| el.bg(accent).text_color(accent_fg))
                                    .when(!is_selected, |el| {
                                        el.text_color(fg).hover(|el| el.bg(hover_bg))
                                    })
                                    .child(theme_name.clone())
                                    .on_click(move |_, window, cx| {
                                        let name = name_for_click.clone();
                                        cx.update_global::<Settings, _>(|settings, _| {
                                            if is_dark {
                                                settings.theme.dark = name;
                                            } else {
                                                settings.theme.light = name;
                                            }
                                            settings.save();
                                        });
                                        apply_theme(window, cx);
                                    })
                            })),
                    )
            })
    }

    fn render_editor_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = cx.global::<Settings>().clone();

        let fg = cx.theme().foreground;
        let muted_fg = cx.theme().muted_foreground;
        let secondary_fg = cx.theme().secondary_foreground;
        let border = cx.theme().border;

        let editor_font_control = NumberInput::new(&self.editor_font_size_input)
            .small()
            .into_any_element();
        let zoom_control = NumberInput::new(&self.zoom_input)
            .small()
            .into_any_element();
        let block_cards = self.render_block_cards(&settings, cx).into_any_element();

        v_flex()
            .gap_1()
            .child(Self::render_section_title("Editor", fg))
            .child(Self::render_sub_header("Font", secondary_fg))
            .child(Self::render_setting_row(
                "Editor Font Size",
                "Font size for the document editor (10-32).",
                editor_font_control,
                fg,
                muted_fg,
                border,
            ))
            .child(Self::render_setting_row(
                "Zoom",
                "Zoom level for the editor (0.5-2.0).",
                zoom_control,
                fg,
                muted_fg,
                border,
            ))
            .child(
                div()
                    .pt_4()
                    .child(Self::render_sub_header("Blocks", secondary_fg)),
            )
            .child(
                Label::new("Enable or disable block types in the slash menu.")
                    .text_xs()
                    .text_color(muted_fg),
            )
            .child(div().pt_2().child(block_cards))
    }

    fn render_block_cards(&self, settings: &Settings, cx: &mut Context<Self>) -> impl IntoElement {
        let disabled = settings.editor.disabled_blocks.clone();
        let border_color = cx.theme().border;
        let bg_color = cx.theme().secondary;
        let fg_color = cx.theme().foreground;

        v_flex().gap_2().children(BLOCK_TYPES.iter().map({
            let disabled = disabled.clone();
            move |block| {
                let is_enabled = !disabled.contains(&block.id.to_string());
                let block_id = block.id.to_string();

                h_flex()
                    .w_full()
                    .justify_between()
                    .items_center()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .border_1()
                    .border_color(border_color)
                    .bg(bg_color)
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                gpui_component::Icon::default()
                                    .path(block.icon_path)
                                    .size_4()
                                    .text_color(fg_color),
                            )
                            .child(
                                Label::new(block.label.to_string())
                                    .text_sm()
                                    .text_color(fg_color),
                            ),
                    )
                    .child(
                        Switch::new(SharedString::from(format!("block-{}", block_id)))
                            .checked(is_enabled)
                            .small()
                            .on_click(cx.listener({
                                let block_id = block_id.clone();
                                move |_, checked, _, cx| {
                                    let block_id = block_id.clone();
                                    cx.update_global::<Settings, _>(|settings, _| {
                                        if *checked {
                                            settings
                                                .editor
                                                .disabled_blocks
                                                .retain(|b| b != &block_id);
                                        } else {
                                            settings.editor.disabled_blocks.push(block_id);
                                        }
                                        settings.save();
                                    });
                                    cx.notify();
                                }
                            })),
                    )
            }
        }))
    }

    fn render_content(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let content = div().flex_1().px_6().pt_2().pb_6();

        match self.active_section {
            SettingsSection::Appearance => content.child(self.render_appearance_section(cx)),
            SettingsSection::Editor => content.child(self.render_editor_section(cx)),
        }
    }
}

impl Render for SettingsWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg = cx.theme().background;

        h_flex()
            .size_full()
            .bg(bg)
            .child(self.render_sidebar(cx))
            .child(
                v_flex()
                    .flex_1()
                    .h_full()
                    .min_w_0()
                    .pt_8()
                    .px_6()
                    .child(self.render_header(cx))
                    .child(self.render_content(cx)),
            )
    }
}

#[derive(IntoElement)]
struct ThemeDropdownTrigger {
    id: ElementId,
    label: String,
    muted_fg: Hsla,
    fg: Hsla,
    input_border: Hsla,
    ring: Hsla,
    bg: Hsla,
    radius: Pixels,
    selected: bool,
}

impl gpui_component::Selectable for ThemeDropdownTrigger {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl RenderOnce for ThemeDropdownTrigger {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_between()
            .w(px(200.))
            .px_2()
            .py_1()
            .rounded(self.radius)
            .border_1()
            .border_color(self.input_border)
            .bg(self.bg)
            .text_sm()
            .text_color(self.fg)
            .cursor_pointer()
            .hover(|el| el.border_color(self.ring))
            .child(
                div()
                    .flex_none()
                    .line_height(relative(1.))
                    .child(self.label),
            )
            .child(
                Icon::new(IconName::ChevronDown)
                    .xsmall()
                    .text_color(self.muted_fg),
            )
    }
}

pub struct SettingsDialog;

impl SettingsDialog {
    pub fn open(_window: &mut Window, cx: &mut App) {
        SettingsWindow::open(cx);
    }
}
