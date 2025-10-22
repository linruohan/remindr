use anyhow::Error;
use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled,
    Subscription, Window, black, div, transparent_white,
};
use gpui_component::input::{InputEvent, InputState, TextInput};
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_value};
use uuid::Uuid;

use crate::{
    Utils,
    controllers::drag_controller::DragElement,
    entities::ui::elements::{ElementNode, ElementNodeParser, RemindrElement},
    screens::parts::document::DocumentState,
};

pub struct TextElement {
    pub data: TextElementData,
    input_state: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl ElementNodeParser<TextElement> for TextElement {
    fn parse(
        data: Value,
        window: &mut Window,
        cx: &mut Context<Self>,
        state: Entity<DocumentState>,
    ) -> Result<Self, Error> {
        let data = from_value::<TextElementData>(data)?;
        let input_state =
            cx.new(|cx| InputState::new(window, cx).default_value(data.metadata.content.clone()));
        let subscriber = Self::prepare_subscribers(&input_state, state.clone(), window, cx);

        Ok(Self {
            data,
            input_state,
            _subscriptions: vec![subscriber],
        })
    }
}

impl TextElement {
    pub fn new(
        id: Uuid,
        window: &mut Window,
        cx: &mut Context<Self>,
        state: Entity<DocumentState>,
    ) -> Self {
        let input_state = cx.new(|cx| InputState::new(window, cx));
        let subscriber = Self::prepare_subscribers(&input_state, state.clone(), window, cx);

        let _subscriptions = vec![subscriber];

        Self {
            data: TextElementData {
                id,
                metadata: Metadata::default(),
            },
            input_state,
            _subscriptions,
        }
    }

    fn prepare_subscribers(
        input_state: &Entity<InputState>,
        state: Entity<DocumentState>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Subscription {
        cx.subscribe_in(&input_state, window, {
            move |this, input_state, ev: &InputEvent, window, cx| match ev {
                InputEvent::Change => {
                    Self::on_change(&mut this.data, input_state, &state, window, cx)
                }
                InputEvent::PressEnter { .. } => {
                    Self::on_press_enter(&this.data, &state, window, cx)
                }
                _ => {}
            }
        })
    }

    fn on_change(
        data: &mut TextElementData,
        input_state: &Entity<InputState>,
        state: &Entity<DocumentState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let value = input_state.read(cx).value();

        if data.metadata.content.is_empty() && value.is_empty() {
            let elements_rc_clone = state.read(cx).elements.clone();
            let index = {
                let elements_guard = elements_rc_clone.borrow();
                elements_guard
                    .iter()
                    .position(|e| e.id == data.id)
                    .unwrap_or_default()
            };

            {
                let mut elements = elements_rc_clone.borrow_mut();
                if elements.len() > 1 {
                    elements.remove(index);

                    let previous_element = elements.get(index.saturating_sub(1));
                    if let Some(node) = previous_element {
                        match node.element.read(cx).child.clone() {
                            RemindrElement::Text(element) => {
                                element.update(cx, |this, cx| {
                                    this.focus(window, cx);
                                });
                            }
                        }
                    }
                }
            }
        } else {
            data.metadata.content = value;
        }

        cx.notify()
    }

    fn on_press_enter(
        data: &TextElementData,
        state: &Entity<DocumentState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let id = Utils::generate_uuid();
        let elements_rc_clone = state.read(cx).elements.clone();

        let insertion_index = {
            let elements_guard = elements_rc_clone.borrow();
            elements_guard
                .iter()
                .position(|e| e.id == data.id)
                .map(|idx| idx + 1)
                .unwrap_or_default()
        };

        let text_element = cx.new(|cx| TextElement::new(id, window, cx, state.clone()));

        let element = RemindrElement::Text(text_element.clone());
        let element = cx.new(|_| DragElement::new(id, state.clone(), element));
        let node = ElementNode::with_id(id, element);

        {
            let mut elements = elements_rc_clone.borrow_mut();
            elements.insert(insertion_index, node);
        }

        text_element.update(cx, |this, cx| {
            this.focus(window, cx);
        });
    }

    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.input_state.update(cx, |element, cx| {
            element.focus(window, cx);
        });
    }
}

impl Render for TextElement {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .text_color(black())
            .text_xs()
            .child(
                TextInput::new(&self.input_state)
                    .bordered(false)
                    .bg(transparent_white())
                    .text_lg()
                    .text_color(black()),
            )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextElementData {
    pub id: Uuid,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    content: SharedString,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            content: SharedString::new(""),
        }
    }
}
