use std::cell::RefCell;
use std::rc::Rc;

use gpui::{Context, DragMoveEvent, Entity, Pixels, Point, Position, Window, div, prelude::*, rgb};
use gpui_component::ActiveTheme;

use crate::{
    Utils,
    controllers::drag_controller::{DragController, DragElement},
    entities::elements::{Element, ElementNode, text_element::TextElement},
};

pub struct DocumentState {
    pub elements: Rc<RefCell<Vec<ElementNode>>>,
    pub drag_controller: Rc<RefCell<DragController>>,
    pub pointer_position: Option<Point<Pixels>>,
}

pub struct Document {
    state: Entity<DocumentState>,
}

impl Document {
    pub fn new(window: &mut Window, ctx: &mut Context<Document>) -> Self {
        let state = ctx.new(|_| DocumentState {
            elements: Rc::new(RefCell::new(Vec::new())),
            drag_controller: Rc::new(RefCell::new(DragController::default())),
            pointer_position: None,
        });

        let elements = state.read(ctx).elements.clone();

        let id = Utils::generate_uuid();

        let drag_info = ctx.new(|ctx| TextElement::new(id, window, ctx, state.clone()));
        let drag_element = ctx.new(|_| DragElement::new(id, state.clone(), drag_info));
        let element_node = ElementNode::with_id(id, Element::Text(drag_element));

        elements.borrow_mut().push(element_node);

        Self { state }
    }
}

impl Render for Document {
    fn render(&mut self, _: &mut Window, ctx: &mut Context<Self>) -> impl IntoElement {
        let elements = self.state.read(ctx).elements.clone();
        let controller = self.state.read(ctx).drag_controller.clone();

        div()
            .flex_1()
            .bg(ctx.theme().background.opacity(0.8))
            .cursor_grab()
            .on_drag_move(
                ctx.listener(move |_, event: &DragMoveEvent<TextElement>, _, ctx| {
                    let is_outside = controller.borrow_mut().on_outside(event);
                    if is_outside {
                        ctx.notify();
                    }
                }),
            )
            .children(elements.borrow().iter().map(|node| {
                div().child(match node.element.clone() {
                    Element::Text(element) => element,
                })
            }))
    }
}
