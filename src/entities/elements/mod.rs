use anyhow::Error;
use gpui::{Context, Entity, IntoElement, Render, Window};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    Utils, controllers::drag_controller::DragElement,
    entities::elements::text_element::TextElement, screens::parts::document::DocumentState,
};

pub mod text_element;

#[derive(Clone)]
pub enum Element {
    Text(Entity<TextElement>),
}

impl Render for Element {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        match &self {
            Element::Text(element) => element.clone(),
        }
    }
}

pub struct ElementNode {
    pub id: Uuid,
    pub element: Entity<DragElement>,
}

impl ElementNode {
    pub fn new(element: Entity<DragElement>) -> Self {
        Self {
            id: Utils::generate_uuid(),
            element,
        }
    }

    pub fn with_id(id: Uuid, element: Entity<DragElement>) -> Self {
        Self { id, element }
    }
}

pub trait AbstractElementNode<T> {
    fn parse(
        payload: Value,
        window: &mut Window,
        ctx: &mut Context<T>,
        state: Entity<DocumentState>,
    ) -> Result<Self, Error>
    where
        Self: Sized;
}
