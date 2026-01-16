# Yashiki

macOS tiling window manager written in Rust.

## Project Structure

```
yashiki/          # WM core daemon
yashiki-ipc/      # Shared protocol definitions (commands, layout)
tatami/           # Default tile layout engine (master-stack)
```

Future components:
- `engawa/` - Status bar
- Other layout engines: `rasen` (spiral), `koushi` (grid)

## Architecture

### Thread Model

- **Main thread**: CFRunLoop
  - Accessibility API (AXUIElement, AXObserver)
  - Global hotkeys (CGEventTap)
  - Window operations
- **Tokio runtime** (separate thread):
  - IPC server (Unix Domain Socket)
  - Config file watching
  - State management, layout calculation

### Communication

- macOS → tokio: `tokio::sync::mpsc`
- tokio → macOS: `dispatch::Queue::main().exec_async()`

### Virtual Workspaces (No SIP Required)

Like AeroSpace, uses virtual workspaces instead of macOS native Spaces:
- All windows exist on a single macOS Space
- Workspace switching moves windows off-screen (x = -10000) or shows them
- Only uses public Accessibility API

## Key Features

- **Tag-based workspace management** (like dwm/awesomewm/river)
  - Windows can have multiple tags (bitmask)
  - View any combination of tags
- **External layout engine** (like river)
  - Layout engine is a separate process
  - Communicates via stdin/stdout JSON
  - Users can write custom layout engines

## Layout Protocol

```rust
// yashiki → layout engine
LayoutRequest { output_width, output_height, window_count, main_count, main_ratio }

// layout engine → yashiki
LayoutResponse { windows: Vec<WindowGeometry> }
```

## Code Style

- All code in English
- Minimal comments - only where logic is non-obvious
- No unnecessary comments explaining what the next line does
