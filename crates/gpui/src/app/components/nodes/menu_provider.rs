use gpui::{App, Entity, SharedString, Window};
use std::rc::Rc;

use crate::app::states::node_state::NodeState;

pub type MenuActionCallback = Rc<dyn Fn(&Entity<NodeState>, &mut Window, &mut App)>;

#[derive(Clone)]
pub struct NodeMenuItem {
    pub id: SharedString,
    pub label: SharedString,
    pub icon_path: &'static str,
    pub action: MenuActionCallback,
}

impl NodeMenuItem {
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        icon_path: &'static str,
        action: impl Fn(&Entity<NodeState>, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon_path,
            action: Rc::new(action),
        }
    }
}

pub trait NodeMenuProvider {
    fn menu_items(&self, cx: &App) -> Vec<NodeMenuItem>;
}
