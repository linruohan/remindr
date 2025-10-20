use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled,
    Subscription, Window, black, div, transparent_white,
};
use gpui_component::input::{InputEvent, InputState, TextInput};
use uuid::Uuid;

use crate::{
    Utils, controllers::drag_controller::DragElement, screens::parts::document::DocumentState,
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
        let input_state = ctx.new(|cx| InputState::new(window, cx).placeholder("Enter your name"));

        let _subscriptions = vec![ctx.subscribe_in(&input_state, window, {
            let input_state = input_state.clone();
            move |this, _, ev: &InputEvent, window, ctx| match ev {
                InputEvent::Change => {
                    let value = input_state.read(ctx).value();
                    this.label = value;

                    ctx.notify()
                }
                InputEvent::PressEnter { .. } => {
                    let id = Utils::generate_uuid();
                    let elements_rc_clone = state.read(ctx).elements.clone();
                    let indexed_elements_rc_clone = state.read(ctx).indexed_elements.clone();

                    let insertion_index = {
                        let elements_guard = elements_rc_clone.borrow();
                        elements_guard
                            .iter()
                            .position(|e| e.read(ctx).id == this.id)
                            .map(|idx| idx + 1)
                            .unwrap_or_default()
                    };

                    let text_element =
                        ctx.new(|ctx| TextElement::new(id.clone(), window, ctx, state.clone()));

                    let element =
                        ctx.new(|_ctx| DragElement::new(id, state.clone(), text_element.clone()));

                    {
                        let mut elements = elements_rc_clone.borrow_mut();
                        elements.insert(insertion_index, element.clone());

                        let mut indexed_elements = indexed_elements_rc_clone.borrow_mut();
                        indexed_elements.insert(insertion_index, id);
                    }

                    text_element.update(ctx, |text_element_inner, ctx| {
                        text_element_inner.focus(window, ctx);
                    });
                }
                _ => {}
            }
        })];

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
