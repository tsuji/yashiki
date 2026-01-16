use super::{Tag, Window, WindowId};
use std::collections::HashMap;

pub struct State {
    pub windows: HashMap<WindowId, Window>,
    pub focused: Option<WindowId>,
    pub visible_tags: Tag,
}

impl State {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            focused: None,
            visible_tags: Tag::new(1),
        }
    }

    pub fn visible_windows(&self) -> impl Iterator<Item = &Window> {
        self.windows
            .values()
            .filter(|w| w.tags.intersects(self.visible_tags))
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
