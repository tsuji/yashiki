use anyhow::Result;
use std::io::{self, BufRead, Write};
use yashiki_ipc::layout::{LayoutRequest, LayoutResponse, WindowGeometry};

fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        let request: LayoutRequest = serde_json::from_str(&line)?;
        let response = generate_layout(&request);
        serde_json::to_writer(&mut stdout, &response)?;
        writeln!(stdout)?;
        stdout.flush()?;
    }

    Ok(())
}

fn generate_layout(req: &LayoutRequest) -> LayoutResponse {
    if req.window_count == 0 {
        return LayoutResponse { windows: vec![] };
    }

    let mut windows = Vec::with_capacity(req.window_count as usize);
    let main_count = req.main_count.min(req.window_count);
    let stack_count = req.window_count - main_count;

    let main_width = if stack_count > 0 {
        (req.output_width as f64 * req.main_ratio) as u32
    } else {
        req.output_width
    };
    let stack_width = req.output_width - main_width;

    // Main area
    let main_height = req.output_height / main_count.max(1);
    for i in 0..main_count {
        windows.push(WindowGeometry {
            x: 0,
            y: (i * main_height) as i32,
            width: main_width,
            height: main_height,
        });
    }

    // Stack area
    if stack_count > 0 {
        let stack_height = req.output_height / stack_count;
        for i in 0..stack_count {
            windows.push(WindowGeometry {
                x: main_width as i32,
                y: (i * stack_height) as i32,
                width: stack_width,
                height: stack_height,
            });
        }
    }

    LayoutResponse { windows }
}
