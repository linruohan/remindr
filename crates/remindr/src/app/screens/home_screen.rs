use gpui::{Context, IntoElement, ParentElement, Render, WeakEntity, Window, div};
use gpui_nav::{Screen, ScreenContext};

use crate::app::states::app_state::AppState;

pub struct HomeScreen {
    _ctx: ScreenContext<AppState>,
}

impl Screen for HomeScreen {
    fn id(&self) -> &'static str {
        "home"
    }
}

impl HomeScreen {
    pub fn new(app_state: WeakEntity<AppState>) -> Self {
        Self {
            _ctx: ScreenContext::new(app_state),
        }
    }
}

impl Render for HomeScreen {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child("Home Screen")
    }
}
