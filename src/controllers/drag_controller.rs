use gpui::{
    App, Context, DragMoveEvent, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, black, div, prelude::FluentBuilder, px, rgb, white,
};
use gpui_component::Icon;
use uuid::Uuid;

use crate::screens::parts::document::DocumentState;

#[derive(Clone, PartialEq)]
pub enum MovingElement {
    After,
    Before,
}

#[derive(Clone)]
pub struct DragController {
    pub dragging_id: Option<Uuid>,
    pub hovered_drop_zone: Option<(Uuid, MovingElement)>,
    pub is_dragging: bool,
}

impl Default for DragController {
    fn default() -> Self {
        Self {
            dragging_id: None,
            hovered_drop_zone: None,
            is_dragging: false,
        }
    }
}

impl DragController {
    pub fn start_drag(&mut self, id: Uuid) {
        self.dragging_id = Some(id);
        self.is_dragging = true;
    }

    pub fn stop_drag(&mut self) {
        self.dragging_id = None;
        self.is_dragging = false;
        self.hovered_drop_zone = None;
    }

    pub fn update_hover_zone(
        &mut self,
        id: Uuid,
        mouse_y: f32,
        bounds_top: f32,
        bounds_height: f32,
    ) -> bool {
        let middle_y = bounds_top + bounds_height / 2.0;
        let zone = if mouse_y < middle_y {
            MovingElement::After
        } else {
            MovingElement::Before
        };

        if mouse_y >= bounds_top && mouse_y <= bounds_top + bounds_height {
            if self.hovered_drop_zone != Some((id, zone.clone())) {
                self.hovered_drop_zone = Some((id, zone.clone()));
                return true;
            }
        } else if let Some((i, _)) = self.hovered_drop_zone {
            if i == id {
                self.hovered_drop_zone = None;
                return true;
            }
        }

        false
    }

    pub fn drop_element_by_index<T>(
        &mut self,
        elements: &mut Vec<T>,
        indexed_elements: &mut Vec<Uuid>,
        from_index: usize,
        target_index: usize,
        position: MovingElement,
    ) {
        let element = elements.remove(from_index);
        let indexed_element = indexed_elements.remove(from_index);

        let mut to_index = target_index;

        match position {
            MovingElement::After => {
                if from_index < target_index {
                    to_index = target_index.saturating_sub(1);
                }
            }
            MovingElement::Before => {
                if from_index >= target_index {
                    to_index = target_index + 1;
                }
            }
        }

        let final_index = to_index.clamp(0, elements.len());
        elements.insert(final_index, element);
        indexed_elements.insert(final_index, indexed_element);

        self.stop_drag();
    }

    pub fn on_outside<T>(&mut self, event: &DragMoveEvent<T>) -> bool {
        let mouse_position = event.event.position;
        let bounds = event.bounds;

        let is_outside = mouse_position.x < bounds.origin.x
            || mouse_position.y < bounds.origin.y
            || mouse_position.x > bounds.origin.x + bounds.size.width
            || mouse_position.y > bounds.origin.y + bounds.size.height;

        if is_outside.clone() {
            println!("DragController::on_outside: Mouse is outside bounds. Stopping drag.");
            self.stop_drag();
        } else {
            println!("DragController::on_outside: Mouse is inside bounds.");
        }

        is_outside
    }
}

pub struct DragElement<T: Render> {
    pub id: Uuid,
    pub child: Entity<T>,
    pub state: Entity<DocumentState>,
}

impl<T: Render> DragElement<T> {
    pub fn new(id: Uuid, state: Entity<DocumentState>, child: Entity<T>) -> Self {
        Self { id, child, state }
    }

    fn on_drop(&self, id: Uuid, direction: MovingElement, ctx: &mut Context<Self>) {
        let elements = self.state.read(ctx).elements.clone();
        let indexed_elements = self.state.read(ctx).indexed_elements.clone();

        let controller = self.state.read(ctx).drag_controller.clone();
        let from_id = controller.borrow().dragging_id.unwrap();
        println!("dd -> {}, {}", id, from_id);

        let from_index = indexed_elements
            .borrow()
            .clone()
            .into_iter()
            .position(|e| e == from_id)
            .unwrap();

        let target_index = indexed_elements
            .borrow()
            .clone()
            .into_iter()
            .position(|e| e == id)
            .unwrap();

        controller.borrow_mut().drop_element_by_index(
            &mut elements.borrow_mut(),
            &mut indexed_elements.borrow_mut(),
            from_index,
            target_index,
            direction,
        );
    }
}

impl<T: Render> Render for DragElement<T> {
    fn render(&mut self, _: &mut Window, ctx: &mut Context<Self>) -> impl IntoElement {
        let element = self.child.clone();
        let is_dragging = self.state.read(ctx).drag_controller.borrow().is_dragging;

        let index = element.entity_id();

        div()
            .w_full()
            .h_12()
            .flex()
            .justify_center()
            .items_center()
            .bg(white())
            .hover(|this| this.bg(white().opacity(0.2)))
            .on_drag_move(
                ctx.listener(move |this, event: &DragMoveEvent<Entity<T>>, _, ctx| {
                    let controller = this.state.read(ctx).drag_controller.clone();

                    let bounds = event.bounds;
                    let middle_y = bounds.origin.y + bounds.size.height / 2.0;
                    let mouse_y = event.event.position.y;

                    let is_in_bounds = mouse_y >= bounds.origin.y
                        && mouse_y <= bounds.origin.y + bounds.size.height;

                    if is_in_bounds {
                        let zone = if mouse_y < middle_y {
                            MovingElement::After
                        } else {
                            MovingElement::Before
                        };

                        if controller.borrow().hovered_drop_zone != Some((this.id, zone.clone())) {
                            controller.borrow_mut().hovered_drop_zone =
                                Some((this.id, zone.clone()));
                            ctx.notify();
                        }
                    } else {
                        let mut controller = controller.borrow_mut();
                        if let Some((i, _)) = controller.hovered_drop_zone.clone() {
                            if i == this.id {
                                controller.hovered_drop_zone = None;
                                ctx.notify();
                            }
                        }
                    }
                }),
            )
            .on_mouse_down(
                gpui::MouseButton::Left,
                ctx.listener(move |this, _, _, ctx| {
                    {
                        let mut controller = this.state.read(ctx).drag_controller.borrow_mut();

                        controller.dragging_id = Some(this.id);
                        controller.is_dragging = true;
                    }

                    ctx.notify();
                }),
            )
            .child(
                div()
                    .id(("item", index))
                    .absolute()
                    .left_0()
                    .flex()
                    .justify_center()
                    .items_center()
                    .hover(|this| this.cursor_grab())
                    .text_color(black())
                    .child(Icon::default().path("icons/grip-vertical.svg").size_6())
                    .when(is_dragging, |this| this.cursor_move())
                    .on_drag(
                        element.clone(),
                        move |element, _, _window: &mut Window, _: &mut App| {
                            println!("element {:?}", element);
                            element.clone()
                        },
                    ),
            )
            .child(div().flex_1().ml_10().child(element))
            .when(is_dragging, |this| {
                let top_dropable_zone_element = div()
                    .absolute()
                    .tab_index(2)
                    .w_full()
                    .h_1_2()
                    .top_0()
                    .on_drop(ctx.listener(move |this, _: &Entity<T>, _, ctx| {
                        this.on_drop(this.id, MovingElement::After, ctx);
                        ctx.notify();
                    }));

                let bottom_dropable_zone_element = div()
                    .absolute()
                    .tab_index(2)
                    .w_full()
                    .h_1_2()
                    .bottom_0()
                    .on_drop(ctx.listener(move |this, _: &Entity<T>, _, ctx| {
                        this.on_drop(this.id, MovingElement::Before, ctx);
                        ctx.notify();
                    }));

                this.child(top_dropable_zone_element)
                    .child(bottom_dropable_zone_element)
            })
            .when_some(
                match self
                    .state
                    .read(ctx)
                    .drag_controller
                    .borrow()
                    .hovered_drop_zone
                    .clone()
                {
                    Some((i, MovingElement::After)) if i == self.id => Some(
                        div()
                            .absolute()
                            .top(px(-1.0))
                            .h(px(2.0))
                            .w_full()
                            .bg(rgb(0xE5EFFC))
                            .tab_index(10),
                    ),
                    Some((i, MovingElement::Before)) if i == self.id => Some(
                        div()
                            .absolute()
                            .bottom(px(-1.0))
                            .h(px(2.0))
                            .w_full()
                            .bg(rgb(0xE5EFFC))
                            .tab_index(10),
                    ),
                    _ => None,
                },
                |this, bar| this.child(bar),
            )
    }
}
