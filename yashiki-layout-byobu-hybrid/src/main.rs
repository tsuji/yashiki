use std::io::{self, BufRead, Write};

use anyhow::Result;

use yashiki_ipc::layout::{LayoutMessage, LayoutResult, WindowGeometry};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Orientation {
    Horizontal,
    Vertical,
}

struct LayoutState {
    main_count: u32,
    main_ratio: f64,
    inner_gap: u32,
    byobu_padding: u32,
    orientation: Orientation,
    main_window_id: Option<u32>,
    focused_window_id: Option<u32>,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            main_count: 1,
            main_ratio: 0.5,
            inner_gap: 0,
            byobu_padding: 30,
            orientation: Orientation::Horizontal,
            main_window_id: None,
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
        // from tatami
        "set-main-ratio" => {
            if let Some(ratio) = args.first().and_then(|s| s.parse::<f64>().ok()) {
                if (0.1..=0.9).contains(&ratio) {
                    state.main_ratio = ratio;
                    return LayoutResult::Ok;
                }
            }
            LayoutResult::Error {
                message: "invalid ratio (must be 0.1-0.9)".to_string(),
            }
        }
        "inc-main-ratio" => {
            let delta = args
                .first()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.05);
            state.main_ratio = (state.main_ratio + delta).min(0.9);
            LayoutResult::Ok
        }
        "dec-main-ratio" => {
            let delta = args
                .first()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.05);
            state.main_ratio = (state.main_ratio - delta).max(0.1);
            LayoutResult::Ok
        }
        "inc-main-count" => {
            state.main_count = state.main_count.saturating_add(1);
            LayoutResult::Ok
        }
        "dec-main-count" => {
            if state.main_count > 1 {
                state.main_count -= 1;
            }
            LayoutResult::Ok
        }
        "set-main-count" => {
            if let Some(count) = args.first().and_then(|s| s.parse::<u32>().ok()) {
                if count >= 1 {
                    state.main_count = count;
                    return LayoutResult::Ok;
                }
            }
            LayoutResult::Error {
                message: "invalid count (must be >= 1)".to_string(),
            }
        }
        "zoom" => {
            let id = args
                .first()
                .and_then(|s| s.parse::<u32>().ok())
                .or(state.focused_window_id);
            if let Some(id) = id {
                state.main_window_id = Some(id);
                LayoutResult::Ok
            } else {
                LayoutResult::Error {
                    message: "no window to zoom (use: zoom <window_id> or focus a window first)"
                        .to_string(),
                }
            }
        }
        // from byobu
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
        // focus handling
        "focus-changed" => {
            if let Some(id) = args.first().and_then(|s| s.parse::<u32>().ok()) {
                state.focused_window_id = Some(id);
                LayoutResult::NeedsRetile
            } else {
                LayoutResult::Error {
                    message: "usage: focus-changed <window_id>".to_string(),
                }
            }
        }
        // other
        "set-inner-gap" => {
            if let Some(gap) = args.first().and_then(|s| s.parse::<u32>().ok()) {
                state.inner_gap = gap;
                return LayoutResult::Ok;
            }
            LayoutResult::Error {
                message: "invalid gap value".to_string(),
            }
        }
        "set-byobu-padding" => {
            if let Some(padding) = args.first().and_then(|s| s.parse::<u32>().ok()) {
                state.byobu_padding = padding;
                return LayoutResult::Ok;
            }
            LayoutResult::Error {
                message: "invalid padding value".to_string(),
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

    // Reorder windows so main_window_id is first (if present)
    let window_ids: Vec<u32> = if let Some(main_id) = state.main_window_id {
        if window_ids.contains(&main_id) {
            let mut reordered = vec![main_id];
            reordered.extend(window_ids.iter().filter(|&&id| id != main_id));
            reordered
        } else {
            window_ids.to_vec()
        }
    } else {
        window_ids.to_vec()
    };

    let window_count = window_ids.len() as u32;
    let inner_gap = state.inner_gap;

    let main_count = state.main_count.min(window_count);
    let stack_count = window_count - main_count;

    // Calculate main/stack widths
    let (main_width, stack_width) = if stack_count > 0 {
        let available_for_windows = width.saturating_sub(inner_gap);
        let mw = (available_for_windows as f64 * state.main_ratio) as u32;
        let sw = available_for_windows.saturating_sub(mw);
        (mw, sw)
    } else {
        (width, 0)
    };

    let mut windows = Vec::with_capacity(window_ids.len());

    // Main area - vertically stacked (from tatami)
    let main_total_gaps = inner_gap.saturating_mul(main_count.saturating_sub(1));
    let main_window_height = height.saturating_sub(main_total_gaps) / main_count.max(1);

    for (i, &window_id) in window_ids.iter().enumerate().take(main_count as usize) {
        let y = i as u32 * (main_window_height + inner_gap);
        let h = if i == main_count as usize - 1 {
            height.saturating_sub(y)
        } else {
            main_window_height
        };
        windows.push(WindowGeometry {
            id: window_id,
            x: 0,
            y: y as i32,
            width: main_width,
            height: h,
        });
    }

    // Stack area - byobu layout
    if stack_count > 0 {
        let stack_x = main_width + inner_gap;
        let stack_ids = &window_ids[main_count as usize..];
        
        // Find focused window within stack
        let focused_in_stack_index = if let Some(focused_id) = state.focused_window_id {
            stack_ids.iter().position(|&id| id == focused_id)
        } else {
            None
        };

        // Reorder stack windows for byobu (focused to end)
        let ordered_stack_ids: Vec<u32> = if let Some(idx) = focused_in_stack_index {
            let mut ids: Vec<u32> = stack_ids.iter().enumerate()
                .filter(|(i, _)| *i != idx)
                .map(|(_, &id)| id)
                .collect();
            ids.push(stack_ids[idx]);
            ids
        } else {
            stack_ids.to_vec()
        };

        let padding = state.byobu_padding;
        let total_offset = padding * (stack_count - 1);

        for (i, &id) in ordered_stack_ids.iter().enumerate() {
            let offset = padding * i as u32;

            match state.orientation {
                Orientation::Horizontal => {
                    windows.push(WindowGeometry {
                        id,
                        x: (stack_x + offset) as i32,
                        y: 0,
                        width: stack_width.saturating_sub(total_offset),
                        height,
                    });
                }
                Orientation::Vertical => {
                    windows.push(WindowGeometry {
                        id,
                        x: stack_x as i32,
                        y: offset as i32,
                        width: stack_width,
                        height: height.saturating_sub(total_offset),
                    });
                }
            }
        }
    }

    windows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_layout_single_window() {
        let state = LayoutState::default();
        let windows = [123];
        let geometries = generate_layout(&state, 1000, 800, &windows);

        assert_eq!(geometries.len(), 1);
        assert_eq!(geometries[0].id, 123);
        assert_eq!(geometries[0].x, 0);
        assert_eq!(geometries[0].y, 0);
        assert_eq!(geometries[0].width, 1000);
        assert_eq!(geometries[0].height, 800);
    }

    #[test]
    fn test_hybrid_layout_two_windows() {
        let mut state = LayoutState::default();
        state.main_ratio = 0.6;
        state.inner_gap = 10;
        state.byobu_padding = 30;
        
        let window_ids = [1, 2];
        let geometries = generate_layout(&state, 1010, 800, &window_ids);

        assert_eq!(geometries.len(), 2);
        
        // Main window (1)
        // available = 1010 - 10 = 1000
        // main_width = 1000 * 0.6 = 600
        assert_eq!(geometries[0].id, 1);
        assert_eq!(geometries[0].x, 0);
        assert_eq!(geometries[0].width, 600);
        assert_eq!(geometries[0].height, 800);

        // Stack window (2) - Byobu
        // stack_x = 600 + 10 = 610
        // stack_width = 1000 - 600 = 400
        // total_offset = 30 * (1 - 1) = 0
        assert_eq!(geometries[1].id, 2);
        assert_eq!(geometries[1].x, 610);
        assert_eq!(geometries[1].width, 400);
        assert_eq!(geometries[1].height, 800);
    }

    #[test]
    fn test_hybrid_layout_three_windows() {
        let mut state = LayoutState::default();
        state.main_ratio = 0.6;
        state.inner_gap = 10;
        state.byobu_padding = 30;
        state.focused_window_id = Some(2);
        
        let window_ids = [1, 2, 3];
        let geometries = generate_layout(&state, 1010, 800, &window_ids);

        assert_eq!(geometries.len(), 3);

        // Main window (1)
        assert_eq!(geometries[0].id, 1);
        assert_eq!(geometries[0].width, 600);

        // Stack windows (2, 3) - Byobu
        // Ordered stack: [3, 2] because 2 is focused
        // Window 3: index 0, offset 0
        assert_eq!(geometries[1].id, 3);
        assert_eq!(geometries[1].x, 610);
        assert_eq!(geometries[1].width, 400 - 30); // stack_width - total_offset

        // Window 2: index 1, offset 30
        assert_eq!(geometries[2].id, 2);
        assert_eq!(geometries[2].x, 610 + 30);
        assert_eq!(geometries[2].width, 400 - 30);
    }
}
