use super::Tag;

pub type WindowId = u64;

#[derive(Debug, Clone)]
pub struct Window {
    pub id: WindowId,
    pub tags: Tag,
    pub title: String,
    pub app_name: String,
    pub frame: Rect,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
