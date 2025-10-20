use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Pixels, Render, Styled, Window, div,
    rgb,
};

use crate::screens::parts::document::Document;

pub struct MainScreen {
    document: Entity<Document>,
}

impl MainScreen {
    pub fn new(window: &mut Window, ctx: &mut Context<Self>) -> Self {
        let document = ctx.new(|ctx| Document::new(window, ctx));
        Self { document }
    }
}

impl Render for MainScreen {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .h_full()
            .flex()
            .child(div().w(Pixels::from(240.0)).bg(rgb(0xebebeb)))
            .child(self.document.clone())
    }
}
