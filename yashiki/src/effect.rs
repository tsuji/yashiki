use crate::core::{Rect, WindowMove};
use crate::macos::DisplayId;
use yashiki_ipc::{Response, WindowGeometry};

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    ApplyWindowMoves(Vec<WindowMove>),
    ApplyLayout {
        display_id: DisplayId,
        frame: Rect,
        geometries: Vec<WindowGeometry>,
    },
    FocusWindow {
        window_id: u32,
        pid: i32,
    },
    MoveWindowToPosition {
        window_id: u32,
        pid: i32,
        x: i32,
        y: i32,
    },
    Retile,
    RetileDisplay(DisplayId),
    RetileDisplays(Vec<DisplayId>),
    SendLayoutCommand {
        cmd: String,
        args: Vec<String>,
    },
    ExecCommand(String),
    FocusVisibleWindowIfNeeded,
    UpdateWindowOrder {
        display_id: DisplayId,
        window_ids: Vec<u32>,
    },
}

pub struct CommandResult {
    pub response: Response,
    pub effects: Vec<Effect>,
}

impl CommandResult {
    pub fn ok() -> Self {
        Self {
            response: Response::Ok,
            effects: vec![],
        }
    }

    pub fn ok_with_effects(effects: Vec<Effect>) -> Self {
        Self {
            response: Response::Ok,
            effects,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            response: Response::Error {
                message: message.into(),
            },
            effects: vec![],
        }
    }

    pub fn with_response(response: Response) -> Self {
        Self {
            response,
            effects: vec![],
        }
    }
}
