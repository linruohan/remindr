use anyhow::Error;
use gpui::{Context, Entity, IntoElement, Render, Window};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    Utils, controllers::drag_controller::DragElement,
    entities::ui::elements::text::text_element::TextElement,
    screens::parts::document::DocumentState,
};

pub mod text;

#[derive(Clone)]
pub enum RemindrElement {
    Text(Entity<TextElement>),
}

impl Render for RemindrElement {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        match &self {
            RemindrElement::Text(element) => element.clone(),
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

pub trait ElementNodeParser<T> {
    fn parse(
        payload: Value,
        window: &mut Window,
        ctx: &mut Context<T>,
        state: Entity<DocumentState>,
    ) -> Result<Self, Error>
    where
        Self: Sized;
}
