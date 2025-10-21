use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled,
    Subscription, Window, black, div, transparent_white,
};
use gpui_component::input::{InputEvent, InputState, TextInput};
use uuid::Uuid;

use crate::{
    Utils,
    controllers::drag_controller::DragElement,
    entities::elements::{Element, ElementNode},
    screens::parts::document::DocumentState,
};

pub struct TextElement {
    pub id: Uuid,
    input_state: Entity<InputState>,
    label: SharedString,
    _subscriptions: Vec<Subscription>,
}

impl TextElement {
    pub fn new(
        id: Uuid,
        window: &mut Window,
        ctx: &mut Context<Self>,
        state: Entity<DocumentState>,
    ) -> Self {
        let input_state = ctx.new(|cx| InputState::new(window, cx));

        let subscriber = ctx.subscribe_in(&input_state, window, {
            move |this, input_state, ev: &InputEvent, window, ctx| match ev {
                InputEvent::Change => {
                    let value = input_state.read(ctx).value();
                    this.label = value;

                    if this.label.is_empty() {
                        let elements_rc_clone = state.read(ctx).elements.clone();
                        let index = {
                            let elements_guard = elements_rc_clone.borrow();
                            elements_guard
                                .iter()
                                .position(|e| e.id == this.id)
                                .unwrap_or_default()
                        };

                        {
                            let mut elements = elements_rc_clone.borrow_mut();
                            elements.remove(index);
                        }

                        // {
                        //     let elements_guard = elements_rc_clone.borrow();
                        //     let previous_element = elements_guard.get(index.saturating_sub(1));

                        //     if let Some(previous_element) = previous_element {
                        //         match previous_element {
                        //             Element::Text(element) => {
                        //                 element.focus();
                        //             }
                        //             _ => {}
                        //         }
                        //     }
                        // }
                    };

                    ctx.notify()
                }
                InputEvent::PressEnter { .. } => {
                    let id = Utils::generate_uuid();
                    let elements_rc_clone = state.read(ctx).elements.clone();

                    let insertion_index = {
                        let elements_guard = elements_rc_clone.borrow();
                        elements_guard
                            .iter()
                            .position(|e| e.id == this.id)
                            .map(|idx| idx + 1)
                            .unwrap_or_default()
                    };

                    let text_element =
                        ctx.new(|ctx| TextElement::new(id, window, ctx, state.clone()));

                    let element =
                        ctx.new(|_ctx| DragElement::new(id, state.clone(), text_element.clone()));

                    let node = ElementNode::with_id(id, Element::Text(element));

                    {
                        let mut elements = elements_rc_clone.borrow_mut();
                        elements.insert(insertion_index, node);
                    }

                    text_element.update(ctx, |text_element_inner, ctx| {
                        text_element_inner.focus(window, ctx);
                    });
                }
                _ => {}
            }
        });

        let _subscriptions = vec![subscriber];

        Self {
            id,
            label: SharedString::new("".to_string()),
            input_state,
            _subscriptions,
        }
    }

    pub fn focus(&self, window: &mut Window, ctx: &mut Context<Self>) {
        self.input_state.update(ctx, |element, ctx| {
            element.focus(window, ctx);
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
                    .text_color(black()),
            )
    }
}
