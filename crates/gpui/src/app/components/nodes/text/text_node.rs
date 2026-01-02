use std::f32::INFINITY;

use anyhow::{Error, Ok};
use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState};
use serde_json::{Value, from_value};
use uuid::Uuid;

use crate::app::{
    components::{
        nodes::{
            element::{NodePayload, RemindrElement},
            menu_provider::{NodeMenuItem, NodeMenuProvider},
            text::data::{TextMetadata, TextNodeData},
            textual_node::{SlashMenuNode, TextualNode, TextualNodeDelegate, TextualNodeEvent},
        },
        slash_menu::{SlashMenu, SlashMenuDismissEvent},
    },
    states::{document_state::DocumentState, node_state::NodeState},
};

pub struct TextNode {
    pub state: Entity<NodeState>,
    pub data: TextNodeData,
    pub input_state: Entity<InputState>,
    menu: Entity<SlashMenu>,
    is_focus: bool,
}

impl TextNode {
    pub fn parse(
        data: &Value,
        state: &Entity<NodeState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<Self, Error> {
        let data = from_value::<TextNodeData>(data.clone())?;

        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(data.metadata.content.clone())
                .auto_grow(1, INFINITY as usize)
                .soft_wrap(true)
        });

        cx.subscribe_in(&input_state, window, {
            move |this, _, ev: &InputEvent, window, cx| match ev {
                InputEvent::Focus => this.handle_focus(window, cx),
                InputEvent::Blur => this.handle_blur(window, cx),
                InputEvent::Change => this.handle_input_change(window, cx),
                InputEvent::PressEnter { .. } => {
                    this.on_textual_event(TextualNodeEvent::Enter, window, cx);
                }
            }
        })
        .detach();

        let menu = cx.new(|cx| SlashMenu::new(data.id, state, window, cx));

        cx.subscribe_in(&menu, window, {
            move |this, _, event: &SlashMenuDismissEvent, window, cx| {
                if event.restore_focus {
                    let input_state = this.input_state.clone();
                    cx.defer_in(window, move |_, window, cx| {
                        input_state.update(cx, |element, cx| {
                            element.focus(window, cx);
                        });
                    });
                }
            }
        })
        .detach();

        Ok(Self {
            state: state.clone(),
            data,
            input_state,
            menu,
            is_focus: false,
        })
    }

    fn handle_input_change(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input_value = self.input_state.read(cx).value();
        let old_content = self.data.metadata.content.clone();

        if input_value.ends_with('/') && self.is_focus {
            self.on_textual_event(TextualNodeEvent::SlashTyped, window, cx);
        }

        if old_content.is_empty() && input_value.is_empty() {
            self.on_textual_event(TextualNodeEvent::Empty, window, cx);
        } else {
            // Update content and emit change event
            self.data.metadata.content = input_value.clone();
            self.on_textual_event(TextualNodeEvent::Change(input_value), window, cx);
        }
    }
}

impl TextualNode for TextNode {
    fn input_state(&self) -> &Entity<InputState> {
        &self.input_state
    }

    fn node_state(&self) -> &Entity<NodeState> {
        &self.state
    }

    fn node_id(&self) -> Uuid {
        self.data.id
    }

    fn content(&self) -> SharedString {
        self.data.metadata.content.clone()
    }

    fn set_content(&mut self, content: SharedString) {
        self.data.metadata.content = content;
    }

    fn is_focused(&self) -> bool {
        self.is_focus
    }

    fn set_focused(&mut self, focused: bool) {
        self.is_focus = focused;
    }
}

impl SlashMenuNode for TextNode {
    fn slash_menu(&self) -> &Entity<SlashMenu> {
        &self.menu
    }
}

impl TextualNodeDelegate for TextNode {
    fn on_textual_event(
        &mut self,
        event: TextualNodeEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            TextualNodeEvent::SlashTyped => {
                let menu_open = self.menu.read(cx).open;
                if !menu_open {
                    self.menu.update(cx, |menu, cx| {
                        menu.set_open(true, window, cx);
                    });
                }
            }
            TextualNodeEvent::Empty => {
                let node_id = self.data.id;
                let state = self.state.clone();

                state.update(cx, |state, inner_cx| {
                    if !state.get_nodes().is_empty() {
                        let previous_element = state.get_previous_node(node_id);
                        state.remove_node(node_id);

                        if let Some(previous_element) = previous_element {
                            if let RemindrElement::Text(element) = previous_element.element.clone()
                            {
                                let input = element.read(inner_cx).input_state().clone();
                                input.update(inner_cx, |input, inner_cx| {
                                    input.focus(window, inner_cx);
                                    input.set_cursor_position(
                                        gpui_component::input::Position::new(u32::MAX, u32::MAX),
                                        window,
                                        inner_cx,
                                    );
                                });
                            }

                            if let RemindrElement::Heading(element) =
                                previous_element.element.clone()
                            {
                                let input = element.read(inner_cx).input_state().clone();
                                input.update(inner_cx, |input, inner_cx| {
                                    input.focus(window, inner_cx);
                                    input.set_cursor_position(
                                        gpui_component::input::Position::new(u32::MAX, u32::MAX),
                                        window,
                                        inner_cx,
                                    );
                                });
                            }
                        }

                        inner_cx.update_global::<DocumentState, _>(|state, app_cx| {
                            state.mark_changed(window, app_cx);
                        });
                    }
                });
            }
            TextualNodeEvent::Enter => {
                if self.menu.read(cx).open {
                    return;
                }

                self.input_state.update(cx, |state, cx| {
                    let value = state.value();
                    state.set_value(value.trim().to_string(), window, cx);
                });

                self.is_focus = false;

                self.state.update(cx, |state, cx| {
                    state.insert_node_after(
                        self.data.id,
                        &RemindrElement::create_node(
                            NodePayload::Text((TextMetadata::default(), true)),
                            &self.state,
                            window,
                            cx,
                        ),
                    );
                });

                cx.update_global::<DocumentState, _>(|state, app_cx| {
                    state.mark_changed(window, app_cx);
                });
            }
            TextualNodeEvent::Change(_) => {
                cx.update_global::<DocumentState, _>(|state, app_cx| {
                    state.mark_changed(window, app_cx);
                });
            }
            TextualNodeEvent::Focus | TextualNodeEvent::Blur => {
                // No additional handling needed for focus/blur
            }
            TextualNodeEvent::Backspace | TextualNodeEvent::Delete => {
                // Reserved for future use
            }
        }
    }
}

impl NodeMenuProvider for TextNode {
    fn menu_items(&self, _cx: &App) -> Vec<NodeMenuItem> {
        vec![]
    }
}

impl Render for TextNode {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .min_w(px(820.0))
            .w_full()
            .child(
                Input::new(&self.input_state)
                    .bordered(false)
                    .bg(transparent_white()),
            )
            .child(self.menu.clone())
    }
}
