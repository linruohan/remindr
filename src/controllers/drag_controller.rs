use gpui::{
    App, Context, DragMoveEvent, Entity, InteractiveElement, IntoElement, MouseEvent,
    MouseMoveEvent, ParentElement, Render, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px, rgb, white,
};
use gpui_component::{ActiveTheme, Icon, IconName};
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
        from_index: usize,
        target_index: usize,
        position: MovingElement,
    ) {
        let element = elements.remove(from_index);

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

        let controller = self.state.read(ctx).drag_controller.clone();
        let from_id = controller.borrow().dragging_id.unwrap();

        let from_index = elements
            .borrow()
            .iter()
            .position(|e| e.id == from_id)
            .unwrap();

        let target_index = elements.borrow().iter().position(|e| e.id == id).unwrap();

        controller.borrow_mut().drop_element_by_index(
            &mut elements.borrow_mut(),
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
            .flex()
            .justify_center()
            .items_center()
            .bg(ctx.theme().background)
            .relative()
            .on_mouse_move(ctx.listener(
                move |this: &mut DragElement<T>,
                      event: &MouseMoveEvent,
                      _: &mut Window,
                      cx: &mut Context<DragElement<T>>| {
                    this.state.update(
                        cx,
                        |this: &mut DocumentState, _: &mut Context<DocumentState>| {
                            this.pointer_position = Some(event.position);
                        },
                    );
                },
            ))
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
                    .absolute()
                    .left_0()
                    .flex()
                    .gap_1()
                    .child(
                        div()
                            .id(("add", index))
                            .size_6()
                            .hover(|this| {
                                this.bg(ctx.theme().background.opacity(0.3)).cursor_grab()
                            })
                            .flex()
                            .justify_center()
                            .items_center()
                            .child(
                                Icon::new(IconName::Plus)
                                    .size_5()
                                    .text_color(ctx.theme().accent_foreground.opacity(0.5)),
                            )
                            .when(is_dragging, |this| this.cursor_move()),
                    )
                    .child(
                        div()
                            .id(("drag", index))
                            .size_6()
                            .hover(|this| {
                                this.bg(ctx.theme().background.opacity(0.3)).cursor_grab()
                            })
                            .flex()
                            .justify_center()
                            .items_center()
                            .child(
                                Icon::default()
                                    .path("icons/grip-vertical.svg")
                                    .size_5()
                                    .text_color(ctx.theme().accent_foreground.opacity(0.5)),
                            )
                            .when(is_dragging, |this| this.cursor_move())
                            .on_drag(
                                element.clone(),
                                move |element, _, _window: &mut Window, _: &mut App| {
                                    element.clone()
                                },
                            ),
                    ),
            )
            .child(
                div()
                    .relative()
                    .flex_1()
                    .ml_10()
                    .child(element)
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
                                    .top(px(-2.0))
                                    .h(px(4.0))
                                    .w_full()
                                    .bg(ctx.theme().accent_foreground.opacity(0.5))
                                    .tab_index(10),
                            ),
                            Some((i, MovingElement::Before)) if i == self.id => Some(
                                div()
                                    .absolute()
                                    .bottom(px(-2.0))
                                    .h(px(4.0))
                                    .w_full()
                                    .bg(ctx.theme().accent_foreground.opacity(0.5))
                                    .tab_index(10),
                            ),
                            _ => None,
                        },
                        |this, bar| this.child(bar),
                    ),
            )
    }
}
