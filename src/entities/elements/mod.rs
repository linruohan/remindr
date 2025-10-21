use gpui::Entity;
use uuid::Uuid;

use crate::{
    Utils, controllers::drag_controller::DragElement, entities::elements::text_element::TextElement,
};

pub mod text_element;

#[derive(Clone)]
pub enum Element {
    Text(Entity<DragElement<TextElement>>),
}

pub struct ElementNode {
    pub id: Uuid,
    pub element: Element,
}

impl ElementNode {
    pub fn new(element: Element) -> Self {
        Self {
            id: Utils::generate_uuid(),
            element,
        }
    }

    pub fn with_id(id: Uuid, element: Element) -> Self {
        Self { id, element }
    }
}
