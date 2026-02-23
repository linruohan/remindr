use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Context, Corner, EmptyView, Entity, FocusHandle, Focusable, Hsla,
    InteractiveElement, IntoElement, KeyDownEvent, MouseButton, ParentElement, Render, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use gpui_component::{ActiveTheme, Icon, Selectable, label::Label, popover::Popover};
use uuid::Uuid;

use crate::app::{
    components::{node_renderer::DraggableInfo, nodes::menu_provider::NodeMenuItem},
    states::node_state::NodeState,
};

const DESTRUCTIVE_COLOR: Hsla = Hsla {
    h: 0.0,
    s: 0.84,
    l: 0.60,
    a: 1.0,
};

pub struct NodeConfigMenu {
    pub related_id: Uuid,
    pub state: Entity<NodeState>,
    pub dragged_info: DraggableInfo,
    pub open: bool,
    pub focus_handle: FocusHandle,
}

impl NodeConfigMenu {
    pub fn new(related_id: Uuid, state: &Entity<NodeState>, cx: &mut Context<Self>) -> Self {
        let dragged_info = DraggableInfo { id: related_id };

        Self {
            related_id,
            state: state.clone(),
            open: false,
            dragged_info,
            focus_handle: cx.focus_handle(),
        }
    }

    fn set_open(&mut self, open: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.open = open;
        if open {
            let focus_handle = self.focus_handle.clone();
            self.state.update(cx, |_, cx| {
                focus_handle.focus(window, cx);
            });
        }
        cx.notify();
    }

    fn delete_node(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |state, _| {
            state.remove_node(self.related_id);
        });
        self.open = false;
        cx.notify();
    }

    fn render_section_label(
        &self,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div().px_2().py_1().child(
            Label::new(label)
                .text_xs()
                .text_color(cx.theme().muted_foreground),
        )
    }

    fn render_delete_item(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("delete-node")
            .flex()
            .items_center()
            .gap_2()
            .w_full()
            .px_2()
            .py_0p5()
            .rounded_md()
            .cursor_pointer()
            .hover(|this| this.bg(DESTRUCTIVE_COLOR.opacity(0.15)))
            .on_click(cx.listener(|this, _, window, cx| {
                this.delete_node(window, cx);
            }))
            .child(
                Icon::default()
                    .path("icons/trash-2.svg")
                    .size_4()
                    .text_color(DESTRUCTIVE_COLOR),
            )
            .child(Label::new("Delete").text_sm().text_color(DESTRUCTIVE_COLOR))
    }
}

impl Focusable for NodeConfigMenu {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for NodeConfigMenu {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let node_menu_items: Vec<NodeMenuItem> = self
            .state
            .read(cx)
            .get_current_nodes(self.related_id)
            .map(|node| node.element.menu_items(cx))
            .unwrap_or_default();

        let has_node_items = !node_menu_items.is_empty();
        let is_dragging = self.state.read(cx).is_dragging;

        let rendered_items: Vec<NodeMenuItemElement> = node_menu_items
            .into_iter()
            .map(|item| NodeMenuItemElement {
                item,
                state: self.state.clone(),
            })
            .collect();

        let drag_button = div()
            .id(self.related_id)
            .size_6()
            .flex()
            .justify_center()
            .items_center()
            .cursor_pointer()
            .child(
                Icon::default()
                    .path("icons/grip-vertical.svg")
                    .size_5()
                    .text_color(cx.theme().accent_foreground.opacity(0.5)),
            )
            .when(is_dragging, |this| this.cursor_move())
            .on_drag(self.dragged_info.clone(), {
                let state = self.state.clone();
                move |element, _, _window: &mut Window, cx: &mut App| {
                    state.update(cx, |state, _| state.start_drag(element.id));
                    cx.new(|_| EmptyView)
                }
            })
            .on_click(cx.listener(|this, _, window, cx| {
                if !this.state.read(cx).is_dragging {
                    this.set_open(!this.open, window, cx);
                }
            }));

        div()
            .relative()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                if event.keystroke.key.as_str() == "escape" && this.open {
                    this.open = false;
                    cx.notify();
                    cx.stop_propagation();
                }
            }))
            .child(drag_button)
            .child(
                Popover::new("contextual-node-popover")
                    .anchor(Corner::TopRight)
                    .trigger(Empty::default())
                    .open(!is_dragging && self.open)
                    .on_open_change(cx.listener(|this, open: &bool, window, cx| {
                        this.set_open(*open, window, cx);
                    }))
                    .p_1()
                    .w(px(200.0))
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_lg()
                    .shadow_lg()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .when(has_node_items, |el| {
                                el.child(self.render_section_label("Turn into", cx))
                                    .children(rendered_items)
                            })
                            .child(self.render_section_label("Actions", cx))
                            .child(self.render_delete_item(cx)),
                    ),
            )
    }
}

#[derive(IntoElement, Default)]
struct Empty {
    selected: bool,
}

impl Selectable for Empty {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl RenderOnce for Empty {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
    }
}

#[derive(IntoElement)]
struct NodeMenuItemElement {
    item: NodeMenuItem,
    state: Entity<NodeState>,
}

impl RenderOnce for NodeMenuItemElement {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let action = self.item.action.clone();
        let state = self.state.clone();

        div()
            .id(self.item.id.clone())
            .flex()
            .items_center()
            .gap_2()
            .w_full()
            .px_2()
            .py_0p5()
            .rounded_md()
            .cursor_pointer()
            .hover(|this| this.bg(cx.theme().accent.opacity(0.5)))
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                (action)(&state, window, cx);
            })
            .child(
                Icon::default()
                    .path(self.item.icon_path)
                    .size_4()
                    .text_color(cx.theme().foreground),
            )
            .child(
                Label::new(self.item.label.clone())
                    .text_sm()
                    .text_color(cx.theme().foreground),
            )
    }
}
