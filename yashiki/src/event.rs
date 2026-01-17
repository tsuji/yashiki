#[derive(Debug, Clone)]
pub enum Event {
    WindowCreated { pid: i32 },
    WindowDestroyed { pid: i32 },
    FocusedWindowChanged,
    WindowMoved { pid: i32 },
    WindowResized { pid: i32 },
    WindowMiniaturized { pid: i32 },
    WindowDeminiaturized { pid: i32 },
    ApplicationActivated { pid: i32 },
    ApplicationDeactivated,
    ApplicationHidden,
    ApplicationShown,
}
