use super::Tag;
use crate::macos::{Bounds, WindowInfo};

pub type WindowId = u32;

#[derive(Debug, Clone)]
pub struct Window {
    pub id: WindowId,
    pub pid: i32,
    pub tags: Tag,
    pub title: String,
    pub app_name: String,
    pub frame: Rect,
    pub is_minimized: bool,
}

impl Window {
    pub fn from_window_info(info: &WindowInfo, default_tag: Tag) -> Self {
        Self {
            id: info.window_id,
            pid: info.pid,
            tags: default_tag,
            title: info.name.clone().unwrap_or_default(),
            app_name: info.owner_name.clone(),
            frame: Rect::from_bounds(&info.bounds),
            is_minimized: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn from_bounds(bounds: &Bounds) -> Self {
        Self {
            x: bounds.x as i32,
            y: bounds.y as i32,
            width: bounds.width as u32,
            height: bounds.height as u32,
        }
    }
}
