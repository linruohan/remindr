use gpui_nav::Navigator;

pub struct AppState {
    pub navigator: Navigator,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            navigator: Navigator::new(),
        }
    }
}
