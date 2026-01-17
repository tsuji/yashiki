use super::{Rect, Tag, WindowId};
use crate::macos::DisplayId;

#[derive(Debug, Clone)]
pub struct Display {
    pub id: DisplayId,
    pub frame: Rect,
    pub visible_tags: Tag,
    pub previous_visible_tags: Tag,
    pub window_order: Vec<WindowId>,
    pub current_layout: Option<String>,
    pub previous_layout: Option<String>,
}

impl Display {
    pub fn new(id: DisplayId, frame: Rect) -> Self {
        Self {
            id,
            frame,
            visible_tags: Tag::new(1),
            previous_visible_tags: Tag::new(1),
            window_order: Vec::new(),
            current_layout: None,
            previous_layout: None,
        }
    }
}
