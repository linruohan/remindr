use std::cell::RefCell;
use std::rc::Rc;

use gpui::{Context, DragMoveEvent, Entity, Window, div, prelude::*, px};
use gpui_component::ActiveTheme;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    Utils,
    controllers::drag_controller::{DragController, DragElement},
    entities::elements::{AbstractElementNode, Element, ElementNode, text_element::TextElement},
};

pub struct DocumentState {
    pub elements: Rc<RefCell<Vec<ElementNode>>>,
    pub drag_controller: Rc<RefCell<DragController>>,
}

pub struct Document {
    state: Entity<DocumentState>,
}

impl Document {
    pub fn new(entries: Vec<Value>, window: &mut Window, ctx: &mut Context<Document>) -> Self {
        let state = ctx.new(|_| DocumentState {
            elements: Rc::new(RefCell::new(Vec::new())),
            drag_controller: Rc::new(RefCell::new(DragController::default())),
        });

        let elements = state.read(ctx).elements.clone();

        for entry in entries {
            let element_type = entry.get("type").unwrap().as_str().unwrap();
            let id = Uuid::parse_str(entry.get("id").unwrap().as_str().unwrap()).unwrap();

            let element = match element_type {
                "text" => Element::Text(
                    ctx.new(|ctx| TextElement::parse(entry, window, ctx, state.clone()).unwrap()),
                ),
                _ => panic!("Unknown element type"),
            };

            let drag_element = ctx.new(|_| DragElement::new(id, state.clone(), element));
            let element_node = ElementNode::with_id(id, drag_element);

            elements.borrow_mut().push(element_node);
        }

        Self { state }
    }
}

impl Render for Document {
    fn render(&mut self, _: &mut Window, ctx: &mut Context<Self>) -> impl IntoElement {
        let elements = self.state.read(ctx).elements.clone();
        let controller = self.state.read(ctx).drag_controller.clone();

        div()
            .flex()
            .flex_1()
            .justify_center()
            .bg(ctx.theme().background.opacity(0.8))
            .child(
                div()
                    .max_w(px(820.0))
                    .w_full()
                    .on_drag_move(
                        ctx.listener(move |_, event: &DragMoveEvent<Element>, _, ctx| {
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
                            .map(|node| div().child(node.element.clone())),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .children(elements.borrow().iter().map(|node| {
                        div().child(format!(
                            "-> {:?}",
                            match node.element.read(ctx).child.clone() {
                                Element::Text(text) => text.read(ctx).data.clone(),
                            }
                        ))
                    })),
            )
    }
}
