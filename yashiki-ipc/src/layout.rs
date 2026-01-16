use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutRequest {
    pub output_width: u32,
    pub output_height: u32,
    pub window_count: u32,
    pub main_count: u32,
    pub main_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutResponse {
    pub windows: Vec<WindowGeometry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
