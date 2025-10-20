use std::cell::RefCell;
use std::rc::Rc;

use gpui::{Context, DragMoveEvent, Entity, Window, div, prelude::*, rgb};
use uuid::Uuid;

use crate::{
    Utils,
    controllers::drag_controller::{DragController, DragElement},
    entities::elements::text_element::TextElement,
};

pub struct DocumentState {
    pub elements: Rc<RefCell<Vec<Entity<DragElement<TextElement>>>>>,
    pub indexed_elements: Rc<RefCell<Vec<Uuid>>>,
    pub drag_controller: Rc<RefCell<DragController>>,
}

pub struct Document {
    state: Entity<DocumentState>,
}

impl Document {
    pub fn new(window: &mut Window, ctx: &mut Context<Document>) -> Self {
        let state = ctx.new(|_| DocumentState {
            elements: Rc::new(RefCell::new(Vec::new())),
            indexed_elements: Rc::new(RefCell::new(Vec::new())),
            drag_controller: Rc::new(RefCell::new(DragController::default())),
        });

        let elements = state.read(ctx).elements.clone();
        let indexed_elements = state.read(ctx).indexed_elements.clone();

        for _ in 0..5 {
            let id = Utils::generate_uuid();

            let drag_info = ctx.new(|ctx| TextElement::new(id, window, ctx, state.clone()));
            let drag_element = ctx.new(|_| DragElement::new(id, state.clone(), drag_info));

            elements.borrow_mut().push(drag_element);
            indexed_elements.borrow_mut().push(id);
        }

        Self { state }
    }
}

impl Render for Document {
    fn render(&mut self, _: &mut Window, ctx: &mut Context<Self>) -> impl IntoElement {
        let elements = self.state.read(ctx).elements.clone();
        let controller = self.state.read(ctx).drag_controller.clone();

        div()
            .flex_1()
            .bg(rgb(0xded3d3))
            .on_drag_move(
                ctx.listener(move |_, event: &DragMoveEvent<TextElement>, _, ctx| {
                    let is_outside = controller.borrow_mut().on_outside(event);
                    if is_outside {
                        ctx.notify();
                    }
                }),
            )
            .children(
                elements
                    .borrow()
                    .iter()
                    .enumerate()
                    .map(|(_, element)| div().child(element.clone())),
            )
    }
}
