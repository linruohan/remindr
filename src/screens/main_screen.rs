use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Pixels, Render, Styled, Window, div,
};
use gpui_component::ActiveTheme;

use crate::{entities::document_parser::DocumentParser, screens::parts::document::Document};

pub struct MainScreen {
    document: Entity<Document>,
}

impl MainScreen {
    pub fn new(window: &mut Window, ctx: &mut Context<Self>) -> Self {
        let entries = DocumentParser::load_file("artifacts/demo.json");
        let document = ctx.new(|ctx| Document::new(entries, window, ctx));

        Self { document }
    }
}

impl Render for MainScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .h_full()
            .flex()
            .child(div().w(Pixels::from(240.0)).bg(cx.theme().accent))
            .child(self.document.clone())
    }
}
