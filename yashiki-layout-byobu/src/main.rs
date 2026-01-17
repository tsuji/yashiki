use anyhow::Result;
use std::io::{self, BufRead, Write};
use yashiki_ipc::layout::{LayoutMessage, LayoutResult, WindowGeometry};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Orientation {
    Horizontal,
    Vertical,
}

struct LayoutState {
    padding: u32,
    orientation: Orientation,
    focused_window_id: Option<u32>,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            padding: 30,
            orientation: Orientation::Horizontal,
            focused_window_id: None,
        }
    }
}

fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut state = LayoutState::default();

    for line in stdin.lock().lines() {
        let line = line?;
        let msg: LayoutMessage = serde_json::from_str(&line)?;
        let result = handle_message(&mut state, msg);
        serde_json::to_writer(&mut stdout, &result)?;
        writeln!(stdout)?;
        stdout.flush()?;
    }

    Ok(())
}

fn handle_message(state: &mut LayoutState, msg: LayoutMessage) -> LayoutResult {
    match msg {
        LayoutMessage::Layout {
            width,
            height,
            windows,
        } => {
            let geometries = generate_layout(state, width, height, &windows);
            LayoutResult::Layout {
                windows: geometries,
            }
        }
        LayoutMessage::Command { cmd, args } => handle_command(state, &cmd, &args),
    }
}

fn handle_command(state: &mut LayoutState, cmd: &str, args: &[String]) -> LayoutResult {
    match cmd {
        "set-padding" => {
            if let Some(padding) = args.first().and_then(|s| s.parse::<u32>().ok()) {
                state.padding = padding;
                return LayoutResult::Ok;
            }
            LayoutResult::Error {
                message: "invalid padding value".to_string(),
            }
        }
        "inc-padding" => {
            let delta = args
                .first()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(5);
            state.padding = state.padding.saturating_add(delta);
            LayoutResult::Ok
        }
        "dec-padding" => {
            let delta = args
                .first()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(5);
            state.padding = state.padding.saturating_sub(delta);
            LayoutResult::Ok
        }
        "set-orientation" => {
            if let Some(orient) = args.first() {
                match orient.as_str() {
                    "horizontal" | "h" => {
                        state.orientation = Orientation::Horizontal;
                        return LayoutResult::Ok;
                    }
                    "vertical" | "v" => {
                        state.orientation = Orientation::Vertical;
                        return LayoutResult::Ok;
                    }
                    _ => {}
                }
            }
            LayoutResult::Error {
                message: "invalid orientation (use horizontal/h or vertical/v)".to_string(),
            }
        }
        "toggle-orientation" => {
            state.orientation = match state.orientation {
                Orientation::Horizontal => Orientation::Vertical,
                Orientation::Vertical => Orientation::Horizontal,
            };
            LayoutResult::Ok
        }
        "focus-changed" => {
            if let Some(id) = args.first().and_then(|s| s.parse::<u32>().ok()) {
                state.focused_window_id = Some(id);
                LayoutResult::Ok
            } else {
                LayoutResult::Error {
                    message: "usage: focus-changed <window_id>".to_string(),
                }
            }
        }
        _ => LayoutResult::Error {
            message: format!("unknown command: {}", cmd),
        },
    }
}

fn generate_layout(
    state: &LayoutState,
    width: u32,
    height: u32,
    window_ids: &[u32],
) -> Vec<WindowGeometry> {
    if window_ids.is_empty() {
        return vec![];
    }

    let window_count = window_ids.len();

    // Find the focused window index
    let focused_index = if let Some(focused_id) = state.focused_window_id {
        window_ids
            .iter()
            .position(|&id| id == focused_id)
            .unwrap_or(0)
    } else {
        0
    };

    let padding = state.padding;

    window_ids
        .iter()
        .enumerate()
        .map(|(index, &id)| {
            let (left_padding, right_padding) =
                calculate_padding(index, window_count, focused_index, padding);

            match state.orientation {
                Orientation::Horizontal => WindowGeometry {
                    id,
                    x: left_padding as i32,
                    y: 0,
                    width: width.saturating_sub(left_padding + right_padding),
                    height,
                },
                Orientation::Vertical => WindowGeometry {
                    id,
                    x: 0,
                    y: left_padding as i32,
                    width,
                    height: height.saturating_sub(left_padding + right_padding),
                },
            }
        })
        .collect()
}

/// Calculate padding for a window based on its position relative to the focused window.
/// Returns (left/top padding, right/bottom padding) depending on orientation.
fn calculate_padding(
    index: usize,
    window_count: usize,
    focused_index: usize,
    padding: u32,
) -> (u32, u32) {
    let last_index = window_count.saturating_sub(1);

    // Single window: no padding
    if window_count == 1 {
        return (0, 0);
    }

    // First window
    if index == 0 {
        // Adjacent to focused gets extra padding
        if focused_index == 1 {
            return (0, 2 * padding);
        }
        return (0, padding);
    }

    // Last window
    if index == last_index {
        // Adjacent to focused gets extra padding
        if focused_index == last_index.saturating_sub(1) {
            return (2 * padding, 0);
        }
        return (padding, 0);
    }

    // Window immediately before focused
    if index + 1 == focused_index {
        return (0, 2 * padding);
    }

    // Window immediately after focused
    if index == focused_index + 1 {
        return (2 * padding, 0);
    }

    // Default: padding on both sides
    (padding, padding)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_window() {
        let state = LayoutState::default();
        let windows = generate_layout(&state, 1920, 1080, &[1]);
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].x, 0);
        assert_eq!(windows[0].y, 0);
        assert_eq!(windows[0].width, 1920);
        assert_eq!(windows[0].height, 1080);
    }

    #[test]
    fn test_two_windows_horizontal() {
        let mut state = LayoutState::default();
        state.padding = 30;
        state.focused_window_id = Some(1);

        let windows = generate_layout(&state, 1920, 1080, &[1, 2]);
        assert_eq!(windows.len(), 2);

        // Window 1 (focused, first): no left padding, 2x right padding (adjacent to focused)
        // Actually window 1 is focused and first, so it's index 0
        // focused_index = 0, so window at index 1 is adjacent after
        // Window 0 (first): (0, padding) but focused_index=0 doesn't trigger "adjacent to focused"
        // Wait, let me reconsider...

        // Window 0 is focused (index 0)
        // Window 1 is at index 1, which is focused_index + 1, so it gets (2*padding, 0)
        // Window 0: first window, focused_index = 0, so case focused_index == 1 is false
        //   -> (0, padding)
        assert_eq!(windows[0].x, 0);
        assert_eq!(windows[0].width, 1920 - 30);

        // Window 1: index=1, last, focused_index=0
        //   index == last_index: true
        //   focused_index == last_index - 1: 0 == 0? yes
        //   -> (2*padding, 0)
        assert_eq!(windows[1].x, 60);
        assert_eq!(windows[1].width, 1920 - 60);
    }

    #[test]
    fn test_three_windows_middle_focused() {
        let mut state = LayoutState::default();
        state.padding = 30;
        state.focused_window_id = Some(2);

        let windows = generate_layout(&state, 1920, 1080, &[1, 2, 3]);
        // focused_index = 1 (window ID 2)

        // Window 0 (first): focused_index == 1, so (0, 2*padding)
        assert_eq!(windows[0].x, 0);
        assert_eq!(windows[0].width, 1920 - 60);

        // Window 1 (focused, middle): default (padding, padding)
        assert_eq!(windows[1].x, 30);
        assert_eq!(windows[1].width, 1920 - 60);

        // Window 2 (last): focused_index == last_index - 1 = 1, so (2*padding, 0)
        assert_eq!(windows[2].x, 60);
        assert_eq!(windows[2].width, 1920 - 60);
    }

    #[test]
    fn test_vertical_orientation() {
        let mut state = LayoutState::default();
        state.padding = 30;
        state.orientation = Orientation::Vertical;
        state.focused_window_id = Some(1);

        let windows = generate_layout(&state, 1920, 1080, &[1, 2]);

        // Window 0 (first, focused): (0, padding) applied to y/height
        assert_eq!(windows[0].x, 0);
        assert_eq!(windows[0].y, 0);
        assert_eq!(windows[0].width, 1920);
        assert_eq!(windows[0].height, 1080 - 30);

        // Window 1 (last): (2*padding, 0) applied to y/height
        assert_eq!(windows[1].x, 0);
        assert_eq!(windows[1].y, 60);
        assert_eq!(windows[1].width, 1920);
        assert_eq!(windows[1].height, 1080 - 60);
    }

    #[test]
    fn test_focus_changed_command() {
        let mut state = LayoutState::default();
        let result = handle_command(&mut state, "focus-changed", &["42".to_string()]);
        assert!(matches!(result, LayoutResult::Ok));
        assert_eq!(state.focused_window_id, Some(42));
    }

    #[test]
    fn test_set_padding_command() {
        let mut state = LayoutState::default();
        let result = handle_command(&mut state, "set-padding", &["50".to_string()]);
        assert!(matches!(result, LayoutResult::Ok));
        assert_eq!(state.padding, 50);
    }

    #[test]
    fn test_toggle_orientation_command() {
        let mut state = LayoutState::default();
        assert_eq!(state.orientation, Orientation::Horizontal);

        handle_command(&mut state, "toggle-orientation", &[]);
        assert_eq!(state.orientation, Orientation::Vertical);

        handle_command(&mut state, "toggle-orientation", &[]);
        assert_eq!(state.orientation, Orientation::Horizontal);
    }
}
