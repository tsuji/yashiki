use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::process;

const PID_FILE: &str = "/tmp/yashiki.pid";

pub fn check_already_running() -> Option<i32> {
    let path = Path::new(PID_FILE);
    if !path.exists() {
        return None;
    }

    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return None,
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return None;
    }

    let pid: i32 = match contents.trim().parse() {
        Ok(p) => p,
        Err(_) => {
            // Invalid PID file, remove it
            let _ = fs::remove_file(path);
            return None;
        }
    };

    // Check if process is still running
    if is_process_running(pid) {
        Some(pid)
    } else {
        // Stale PID file, remove it
        let _ = fs::remove_file(path);
        None
    }
}

pub fn write_pid() -> std::io::Result<()> {
    let mut file = fs::File::create(PID_FILE)?;
    write!(file, "{}", process::id())?;
    Ok(())
}

pub fn remove_pid() {
    let _ = fs::remove_file(PID_FILE);
}

fn is_process_running(pid: i32) -> bool {
    // On Unix, kill with signal 0 checks if process exists
    unsafe { libc::kill(pid, 0) == 0 }
}
