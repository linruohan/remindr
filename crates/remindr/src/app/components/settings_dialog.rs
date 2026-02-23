use crate::app::{
    apply_theme,
    states::settings_state::{Settings, ThemeMode},
};
use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, BorrowAppContext, Bounds, Context, Corner, ElementId, Entity, Hsla,
    InteractiveElement, IntoElement, ParentElement, Pixels, Render, RenderOnce, SharedString, Size,
    StatefulInteractiveElement, Styled, TitlebarOptions, Window, WindowBounds, WindowId,
    WindowKind, WindowOptions, div, point, px, relative, size,
};
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName, Root, Sizable, StyledExt,
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
use std::sync::Mutex;

/// 步进输入的参数结构体
struct StepInputParams {
    step: f32,
    min: f32,
    max: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsSection {
    Appearance,
    Editor,
    Blocks,
}

struct NodeComponent {
    id: &'static str,
    label: &'static str,
    description: &'static str,
    icon_path: &'static str,
}

const NODE_COMPONENTS: &[NodeComponent] = &[
    NodeComponent {
        id: "text",
        label: "Text",
        description: "Plain text paragraph block.",
        icon_path: "icons/pilcrow.svg",
    },
    NodeComponent {
        id: "heading",
        label: "Heading",
        description: "Section headings with multiple levels.",
        icon_path: "icons/heading-2.svg",
    },
    NodeComponent {
        id: "divider",
        label: "Divider",
        description: "Horizontal separator line.",
        icon_path: "icons/separator-horizontal.svg",
    },
];

struct HeadingLevel {
    id: &'static str,
    label: &'static str,
    icon_path: &'static str,
}

const HEADING_LEVELS: &[HeadingLevel] = &[
    HeadingLevel {
        id: "heading_2",
        label: "Heading 2",
        icon_path: "icons/heading-2.svg",
    },
    HeadingLevel {
        id: "heading_3",
        label: "Heading 3",
        icon_path: "icons/heading-3.svg",
    },
];

static SETTINGS_WINDOW: Mutex<Option<WindowId>> = Mutex::new(None);

pub struct SettingsWindow {
    active_section: SettingsSection,
    ui_font_size_input: Entity<InputState>,
    editor_font_size_input: Entity<InputState>,
    zoom_input: Entity<InputState>,
    h1_font_size_input: Entity<InputState>,
    h2_font_size_input: Entity<InputState>,
    h3_font_size_input: Entity<InputState>,
    text_font_size_input: Entity<InputState>,
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

        let block_sizes = &settings.editor.block_font_sizes;

        let h1_font_size_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            state.set_value(format!("{}", block_sizes.heading_1 as i32), window, cx);
            state
        });

        let h2_font_size_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            state.set_value(format!("{}", block_sizes.heading_2 as i32), window, cx);
            state
        });

        let h3_font_size_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            state.set_value(format!("{}", block_sizes.heading_3 as i32), window, cx);
            state
        });

        let text_font_size_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            state.set_value(format!("{}", block_sizes.text as i32), window, cx);
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
            |this, _, event: &NumberInputEvent, window, cx| {
                let NumberInputEvent::Step(action) = event;
                this.step_input(
                    &this.ui_font_size_input.clone(),
                    action,
                    StepInputParams {
                        step: 1.0,
                        min: 10.0,
                        max: 24.0,
                    },
                    window,
                    cx,
                );
                this.on_ui_font_size_changed(cx);
            },
        )
        .detach();

        cx.subscribe_in(
            &editor_font_size_input,
            window,
            |this, _, event: &NumberInputEvent, window, cx| {
                let NumberInputEvent::Step(action) = event;
                this.step_input(
                    &this.editor_font_size_input.clone(),
                    action,
                    StepInputParams {
                        step: 1.0,
                        min: 10.0,
                        max: 32.0,
                    },
                    window,
                    cx,
                );
                this.on_editor_font_size_changed(cx);
            },
        )
        .detach();

        cx.subscribe_in(
            &zoom_input,
            window,
            |this, _, event: &NumberInputEvent, window, cx| {
                let NumberInputEvent::Step(action) = event;
                this.step_input(
                    &this.zoom_input.clone(),
                    action,
                    StepInputParams {
                        step: 0.1,
                        min: 0.5,
                        max: 2.0,
                    },
                    window,
                    cx,
                );
                this.on_zoom_changed(cx);
            },
        )
        .detach();

        // Block font size subscriptions
        for (input, block_key) in [
            (&h1_font_size_input, "heading_1"),
            (&h2_font_size_input, "heading_2"),
            (&h3_font_size_input, "heading_3"),
            (&text_font_size_input, "text"),
        ] {
            let block_key = block_key.to_string();
            cx.subscribe_in(input, window, {
                let block_key = block_key.clone();
                move |_this, _, event: &InputEvent, _, cx| {
                    if let InputEvent::Change = event {
                        Self::on_block_font_size_changed(&block_key, _this, cx);
                    }
                }
            })
            .detach();

            cx.subscribe_in(input, window, {
                let block_key = block_key.clone();
                move |this, _, event: &NumberInputEvent, window, cx| {
                    let NumberInputEvent::Step(action) = event;
                    let input = match block_key.as_str() {
                        "heading_1" => &this.h1_font_size_input,
                        "heading_2" => &this.h2_font_size_input,
                        "heading_3" => &this.h3_font_size_input,
                        _ => &this.text_font_size_input,
                    }
                    .clone();
                    this.step_input(
                        &input,
                        action,
                        StepInputParams {
                            step: 1.0,
                            min: 8.0,
                            max: 72.0,
                        },
                        window,
                        cx,
                    );
                    Self::on_block_font_size_changed(&block_key, this, cx);
                }
            })
            .detach();
        }

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
            h1_font_size_input,
            h2_font_size_input,
            h3_font_size_input,
            text_font_size_input,
            light_theme_search,
            dark_theme_search,
        }
    }

    fn step_input(
        &self,
        input: &Entity<InputState>,
        action: &gpui_component::input::StepAction,
        params: StepInputParams,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let current: f32 = input.read(cx).value().parse().unwrap_or(0.0);
        let new_value = match action {
            gpui_component::input::StepAction::Increment => (current + params.step).min(params.max),
            gpui_component::input::StepAction::Decrement => (current - params.step).max(params.min),
        };
        let formatted = if params.step < 1.0 {
            format!("{:.1}", new_value)
        } else {
            format!("{}", new_value as i32)
        };
        input.update(cx, |state, cx| {
            state.set_value(formatted, window, cx);
        });
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

    fn on_block_font_size_changed(block_key: &str, this: &Self, cx: &mut Context<Self>) {
        let input = match block_key {
            "heading_1" => &this.h1_font_size_input,
            "heading_2" => &this.h2_font_size_input,
            "heading_3" => &this.h3_font_size_input,
            _ => &this.text_font_size_input,
        };
        let value = input.read(cx).value();
        if let Ok(size) = value.parse::<f32>() {
            let size = size.clamp(8.0, 72.0);
            let block_key = block_key.to_string();
            cx.update_global::<Settings, _>(move |settings, _| {
                match block_key.as_str() {
                    "heading_1" => settings.editor.block_font_sizes.heading_1 = size,
                    "heading_2" => settings.editor.block_font_sizes.heading_2 = size,
                    "heading_3" => settings.editor.block_font_sizes.heading_3 = size,
                    _ => settings.editor.block_font_sizes.text = size,
                }
                settings.save();
            });
        }
    }

    pub fn open(cx: &mut App) {
        // If a settings window already exists, focus it
        if let Some(window_id) = *SETTINGS_WINDOW.lock().unwrap() {
            for window in cx.windows() {
                if window.window_id() == window_id {
                    let _ = window.update(cx, |_, window, _| {
                        window.activate_window();
                    });
                    return;
                }
            }
            // Window no longer exists, clear the handle
            *SETTINGS_WINDOW.lock().unwrap() = None;
        }

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
                }),
                ..Default::default()
            };

            let window = cx
                .open_window(options, |window, cx| {
                    let view = cx.new(|cx| SettingsWindow::new(window, cx));
                    cx.new(|cx| Root::new(view, window, cx))
                })
                .expect("failed to open settings window");

            *SETTINGS_WINDOW.lock().unwrap() = Some(window.window_id());

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
            (SettingsSection::Blocks, "Blocks", "icons/layout-grid.svg"),
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
        let section_title = match self.active_section {
            SettingsSection::Appearance => "Appearance",
            SettingsSection::Editor => "Editor",
            SettingsSection::Blocks => "Blocks",
        };

        h_flex()
            .w_full()
            .justify_between()
            .items_center()
            .pb_4()
            .child(
                Label::new(section_title)
                    .text_sm()
                    .font_semibold()
                    .text_color(cx.theme().foreground),
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

    fn render_number_with_reset(
        &self,
        id: &str,
        input: &Entity<InputState>,
        default_value: f32,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let input_clone = input.clone();
        let current: f32 = input.read(cx).value().parse().unwrap_or(0.0);
        let is_default = (current - default_value).abs() < 0.01;
        let muted_fg = cx.theme().muted_foreground;

        h_flex()
            .gap_1()
            .items_center()
            .child(div().w(px(120.)).child(NumberInput::new(input).small()))
            .child(
                Button::new(SharedString::from(id.to_string()))
                    .xsmall()
                    .ghost()
                    .icon(Icon::new(IconName::Undo2).xsmall().text_color(muted_fg))
                    .disabled(is_default)
                    .tooltip("Reset to default")
                    .on_click(cx.listener(move |_this, _, window, cx| {
                        let formatted = if default_value.fract() == 0.0 {
                            format!("{}", default_value as i32)
                        } else {
                            format!("{:.1}", default_value)
                        };
                        input_clone.update(cx, |state, cx| {
                            state.set_value(formatted, window, cx);
                        });
                    })),
            )
    }

    fn render_appearance_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = cx.global::<Settings>().clone();
        let current_mode = settings.theme.mode;

        let fg = cx.theme().foreground;
        let muted_fg = cx.theme().muted_foreground;
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
        let ui_font_control = self
            .render_number_with_reset("reset-ui-font", &self.ui_font_size_input.clone(), 14.0, cx)
            .into_any_element();

        // -- Theme card --
        let theme_card = v_flex()
            .w_full()
            .p_3()
            .rounded_lg()
            .border_1()
            .border_color(border)
            .gap_3()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size_8()
                            .rounded_md()
                            .bg(border)
                            .child(
                                gpui_component::Icon::default()
                                    .path("icons/palette.svg")
                                    .size_4()
                                    .text_color(fg),
                            ),
                    )
                    .child(
                        v_flex()
                            .gap_0p5()
                            .child(Label::new("Theme").text_sm().font_semibold().text_color(fg))
                            .child(
                                Label::new("Customize the application theme.")
                                    .text_xs()
                                    .text_color(muted_fg),
                            ),
                    ),
            )
            .child(
                v_flex()
                    .gap_0()
                    .child(
                        h_flex()
                            .w_full()
                            .justify_between()
                            .items_center()
                            .py_2()
                            .px_2()
                            .child(Label::new("Theme Mode").text_xs().text_color(fg))
                            .child(theme_mode_control),
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .justify_between()
                            .items_center()
                            .py_2()
                            .px_2()
                            .child(Label::new("Light Theme").text_xs().text_color(fg))
                            .child(light_theme_control),
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .justify_between()
                            .items_center()
                            .py_2()
                            .px_2()
                            .child(Label::new("Dark Theme").text_xs().text_color(fg))
                            .child(dark_theme_control),
                    ),
            );

        // -- Font card --
        let font_card = v_flex()
            .w_full()
            .p_3()
            .rounded_lg()
            .border_1()
            .border_color(border)
            .gap_3()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size_8()
                            .rounded_md()
                            .bg(border)
                            .child(
                                gpui_component::Icon::default()
                                    .path("icons/type.svg")
                                    .size_4()
                                    .text_color(fg),
                            ),
                    )
                    .child(
                        v_flex()
                            .gap_0p5()
                            .child(Label::new("Font").text_sm().font_semibold().text_color(fg))
                            .child(
                                Label::new("Font size for the application interface.")
                                    .text_xs()
                                    .text_color(muted_fg),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .w_full()
                    .justify_between()
                    .items_center()
                    .py_2()
                    .px_2()
                    .child(Label::new("UI Font Size").text_xs().text_color(fg))
                    .child(ui_font_control),
            );

        v_flex().gap_3().child(theme_card).child(font_card)
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
        let fg = cx.theme().foreground;
        let muted_fg = cx.theme().muted_foreground;
        let border = cx.theme().border;

        // -- Font card --
        let font_card = v_flex()
            .w_full()
            .p_3()
            .rounded_lg()
            .border_1()
            .border_color(border)
            .gap_3()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size_8()
                            .rounded_md()
                            .bg(border)
                            .child(
                                gpui_component::Icon::default()
                                    .path("icons/type.svg")
                                    .size_4()
                                    .text_color(fg),
                            ),
                    )
                    .child(
                        v_flex()
                            .gap_0p5()
                            .child(Label::new("Font").text_sm().font_semibold().text_color(fg))
                            .child(
                                Label::new("Base font size and zoom for the editor.")
                                    .text_xs()
                                    .text_color(muted_fg),
                            ),
                    ),
            )
            .child(
                v_flex()
                    .gap_0()
                    .child(self.render_editor_setting_row(
                        "Editor Font Size",
                        &self.editor_font_size_input.clone(),
                        16.0,
                        cx,
                    ))
                    .child(self.render_editor_setting_row(
                        "Zoom",
                        &self.zoom_input.clone(),
                        1.0,
                        cx,
                    )),
            );

        // -- Block Font Sizes card --
        struct BlockFontRow {
            label: &'static str,
            icon_path: &'static str,
        }

        let block_rows = [
            (
                "h1",
                BlockFontRow {
                    label: "Heading 1",
                    icon_path: "icons/heading-1.svg",
                },
                &self.h1_font_size_input,
                30.0,
            ),
            (
                "h2",
                BlockFontRow {
                    label: "Heading 2",
                    icon_path: "icons/heading-2.svg",
                },
                &self.h2_font_size_input,
                24.0,
            ),
            (
                "h3",
                BlockFontRow {
                    label: "Heading 3",
                    icon_path: "icons/heading-3.svg",
                },
                &self.h3_font_size_input,
                20.0,
            ),
            (
                "text",
                BlockFontRow {
                    label: "Text",
                    icon_path: "icons/pilcrow.svg",
                },
                &self.text_font_size_input,
                16.0,
            ),
        ];

        let mut block_list = v_flex().gap_0();
        for (id, row, input, default) in &block_rows {
            let control = self
                .render_number_with_reset(&format!("reset-{}", id), input, *default, cx)
                .into_any_element();

            block_list = block_list.child(
                h_flex()
                    .w_full()
                    .justify_between()
                    .items_center()
                    .py_2()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                gpui_component::Icon::default()
                                    .path(row.icon_path)
                                    .size_3()
                                    .text_color(muted_fg),
                            )
                            .child(Label::new(row.label.to_string()).text_xs().text_color(fg)),
                    )
                    .child(control),
            );
        }

        let block_font_card = v_flex()
            .w_full()
            .p_3()
            .rounded_lg()
            .border_1()
            .border_color(border)
            .gap_3()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size_8()
                            .rounded_md()
                            .bg(border)
                            .child(
                                gpui_component::Icon::default()
                                    .path("icons/text-cursor.svg")
                                    .size_4()
                                    .text_color(fg),
                            ),
                    )
                    .child(
                        v_flex()
                            .gap_0p5()
                            .child(
                                Label::new("Block Font Sizes")
                                    .text_sm()
                                    .font_semibold()
                                    .text_color(fg),
                            )
                            .child(
                                Label::new("Custom font size per block type.")
                                    .text_xs()
                                    .text_color(muted_fg),
                            ),
                    ),
            )
            .child(v_flex().w_full().px_2().child(block_list));

        v_flex().gap_3().child(font_card).child(block_font_card)
    }

    fn render_editor_setting_row(
        &self,
        label: &str,
        input: &Entity<InputState>,
        default_value: f32,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let fg = cx.theme().foreground;

        let control = self
            .render_number_with_reset(
                &format!("reset-{}", label.to_lowercase().replace(' ', "-")),
                input,
                default_value,
                cx,
            )
            .into_any_element();

        h_flex()
            .w_full()
            .justify_between()
            .items_center()
            .py_2()
            .px_2()
            .child(Label::new(label.to_string()).text_xs().text_color(fg))
            .child(control)
    }

    fn render_blocks_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = cx.global::<Settings>().clone();
        let disabled = &settings.editor.disabled_blocks;
        let fg = cx.theme().foreground;
        let muted_fg = cx.theme().muted_foreground;
        let border = cx.theme().border;
        let link_color = cx.theme().link;

        let mut section = v_flex().gap_3();

        for node in NODE_COMPONENTS {
            let is_enabled = !disabled.contains(&node.id.to_string());
            let node_id = node.id.to_string();

            let switch = Switch::new(SharedString::from(format!("node-{}", node.id)))
                .checked(is_enabled)
                .small()
                .on_click(cx.listener({
                    let node_id = node_id.clone();
                    move |_, checked, _, cx| {
                        let node_id = node_id.clone();
                        cx.update_global::<Settings, _>(|settings, _| {
                            if *checked {
                                settings.editor.disabled_blocks.retain(|b| b != &node_id);
                            } else {
                                settings.editor.disabled_blocks.push(node_id);
                            }
                            settings.save();
                        });
                        cx.notify();
                    }
                }));

            let has_font_config = node.id != "divider";

            let mut card =
                v_flex()
                    .w_full()
                    .px_2()
                    .py_3()
                    .rounded_lg()
                    .border_1()
                    .border_color(border)
                    .gap_3()
                    .child(
                        h_flex()
                            .w_full()
                            .justify_between()
                            .items_center()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .size_8()
                                            .rounded_md()
                                            .bg(border)
                                            .child(
                                                gpui_component::Icon::default()
                                                    .path(node.icon_path)
                                                    .size_4()
                                                    .text_color(fg),
                                            ),
                                    )
                                    .child(
                                        v_flex()
                                            .gap_0p5()
                                            .child(
                                                Label::new(node.label.to_string())
                                                    .text_sm()
                                                    .font_semibold()
                                                    .text_color(fg),
                                            )
                                            .child(
                                                Label::new(node.description.to_string())
                                                    .text_xs()
                                                    .text_color(muted_fg),
                                            ),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .when(has_font_config, |el| {
                                        el.child(
                                            Button::new(SharedString::from(format!(
                                                "configure-{}",
                                                node.id
                                            )))
                                            .xsmall()
                                            .ghost()
                                            .label("Configure")
                                            .icon(
                                                Icon::new(IconName::ArrowRight)
                                                    .xsmall()
                                                    .text_color(link_color),
                                            )
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.active_section = SettingsSection::Editor;
                                                cx.notify();
                                            })),
                                        )
                                    })
                                    .when(node.id != "heading", |el| el.child(switch)),
                            ),
                    );

            // Heading: add sub-level toggles
            if node.id == "heading" {
                let mut levels_list = v_flex().gap_0();

                for level in HEADING_LEVELS.iter() {
                    let level_enabled = !disabled.contains(&level.id.to_string());
                    let level_id = level.id.to_string();

                    let level_row = h_flex()
                        .w_full()
                        .justify_between()
                        .items_center()
                        .py_2()
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(
                                    gpui_component::Icon::default()
                                        .path(level.icon_path)
                                        .size_3()
                                        .text_color(muted_fg),
                                )
                                .child(
                                    Label::new(level.label.to_string()).text_xs().text_color(fg),
                                ),
                        )
                        .child(
                            Switch::new(SharedString::from(format!("level-{}", level.id)))
                                .checked(level_enabled)
                                .small()
                                .on_click(cx.listener({
                                    let level_id = level_id.clone();
                                    move |_, checked, _, cx| {
                                        let level_id = level_id.clone();
                                        cx.update_global::<Settings, _>(|settings, _| {
                                            if *checked {
                                                settings
                                                    .editor
                                                    .disabled_blocks
                                                    .retain(|b| b != &level_id);
                                            } else {
                                                settings.editor.disabled_blocks.push(level_id);
                                            }
                                            settings.save();
                                        });
                                        cx.notify();
                                    }
                                })),
                        );

                    levels_list = levels_list.child(level_row);
                }

                card = card.child(
                    v_flex()
                        .w_full()
                        .px_2()
                        .child(
                            Label::new("Levels")
                                .text_xs()
                                .font_semibold()
                                .text_color(muted_fg),
                        )
                        .child(levels_list),
                );
            }

            section = section.child(card);
        }

        section
    }

    fn render_content(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let content = div()
            .flex_1()
            .min_h_0()
            .pt_2()
            .pb_6()
            .overflow_y_scrollbar();

        match self.active_section {
            SettingsSection::Appearance => content.child(self.render_appearance_section(cx)),
            SettingsSection::Editor => content.child(self.render_editor_section(cx)),
            SettingsSection::Blocks => content.child(self.render_blocks_section(cx)),
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
                    .px_3()
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
