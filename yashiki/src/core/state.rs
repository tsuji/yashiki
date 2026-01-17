use super::{Tag, Window, WindowId};
use crate::event::Event;
use crate::macos::{get_on_screen_windows, WindowInfo};
use std::collections::{HashMap, HashSet};

pub struct State {
    pub windows: HashMap<WindowId, Window>,
    pub focused: Option<WindowId>,
    pub visible_tags: Tag,
    default_tag: Tag,
}

impl State {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            focused: None,
            visible_tags: Tag::new(1),
            default_tag: Tag::new(1),
        }
    }

    pub fn visible_windows(&self) -> impl Iterator<Item = &Window> {
        self.windows
            .values()
            .filter(|w| w.tags.intersects(self.visible_tags))
    }

    pub fn sync_all(&mut self) {
        let window_infos = get_on_screen_windows();
        self.sync_with_window_infos(&window_infos);
        tracing::info!("State initialized with {} windows", self.windows.len());
        for window in self.windows.values() {
            tracing::debug!("  - [{}] {} ({})", window.id, window.title, window.app_name);
        }
    }

    pub fn sync_pid(&mut self, pid: i32) {
        let window_infos = get_on_screen_windows();
        let pid_window_infos: Vec<_> = window_infos.iter().filter(|w| w.pid == pid).collect();

        let current_ids: HashSet<WindowId> = self
            .windows
            .values()
            .filter(|w| w.pid == pid)
            .map(|w| w.id)
            .collect();
        let new_ids: HashSet<WindowId> = pid_window_infos.iter().map(|w| w.window_id).collect();

        // Remove windows that no longer exist
        for id in current_ids.difference(&new_ids) {
            if let Some(window) = self.windows.remove(id) {
                tracing::info!(
                    "Window removed: [{}] {} ({})",
                    window.id,
                    window.title,
                    window.app_name
                );
            }
        }

        // Add new windows
        for id in new_ids.difference(&current_ids) {
            if let Some(info) = pid_window_infos.iter().find(|w| w.window_id == *id) {
                let window = Window::from_window_info(info, self.default_tag);
                tracing::info!(
                    "Window added: [{}] {} ({})",
                    window.id,
                    window.title,
                    window.app_name
                );
                self.windows.insert(window.id, window);
            }
        }

        // Update existing windows
        for id in current_ids.intersection(&new_ids) {
            if let Some(info) = pid_window_infos.iter().find(|w| w.window_id == *id) {
                if let Some(window) = self.windows.get_mut(id) {
                    let new_title = info.name.clone().unwrap_or_default();
                    let new_frame = super::Rect::from_bounds(&info.bounds);

                    let title_changed = window.title != new_title;
                    let frame_changed = window.frame.x != new_frame.x
                        || window.frame.y != new_frame.y
                        || window.frame.width != new_frame.width
                        || window.frame.height != new_frame.height;

                    if title_changed || frame_changed {
                        window.title = new_title;
                        window.frame = new_frame;
                        tracing::debug!(
                            "Window updated: [{}] {} ({})",
                            window.id,
                            window.title,
                            window.app_name
                        );
                    }
                }
            }
        }
    }

    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::WindowCreated { pid }
            | Event::WindowDestroyed { pid }
            | Event::WindowMoved { pid }
            | Event::WindowResized { pid }
            | Event::WindowMiniaturized { pid }
            | Event::WindowDeminiaturized { pid } => {
                self.sync_pid(*pid);
            }
            Event::FocusedWindowChanged { pid } => {
                self.sync_pid(*pid);
                self.update_focused_from_pid(*pid);
            }
            Event::ApplicationActivated { pid } => {
                self.update_focused_from_pid(*pid);
            }
            Event::ApplicationDeactivated { .. }
            | Event::ApplicationHidden { .. }
            | Event::ApplicationShown { .. } => {}
        }
    }

    pub fn set_focused(&mut self, window_id: Option<WindowId>) {
        if self.focused != window_id {
            tracing::info!("Focus changed: {:?} -> {:?}", self.focused, window_id);
            self.focused = window_id;
        }
    }

    fn update_focused_from_pid(&mut self, pid: i32) {
        let window_id = self.windows.values().find(|w| w.pid == pid).map(|w| w.id);
        self.set_focused(window_id);
    }

    fn sync_with_window_infos(&mut self, window_infos: &[WindowInfo]) {
        let current_ids: HashSet<WindowId> = self.windows.keys().copied().collect();
        let new_ids: HashSet<WindowId> = window_infos.iter().map(|w| w.window_id).collect();

        // Remove windows that no longer exist
        for id in current_ids.difference(&new_ids) {
            self.windows.remove(id);
        }

        // Add new windows
        for info in window_infos {
            if !self.windows.contains_key(&info.window_id) {
                let window = Window::from_window_info(info, self.default_tag);
                self.windows.insert(window.id, window);
            }
        }

        // Update existing windows
        for info in window_infos {
            if let Some(window) = self.windows.get_mut(&info.window_id) {
                window.title = info.name.clone().unwrap_or_default();
                window.frame = super::Rect::from_bounds(&info.bounds);
            }
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
