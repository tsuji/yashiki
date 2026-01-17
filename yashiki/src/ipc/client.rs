use anyhow::{Context, Result};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use yashiki_ipc::{Command, Response};

const SOCKET_PATH: &str = "/tmp/yashiki.sock";

pub struct IpcClient {
    stream: UnixStream,
}

impl IpcClient {
    pub fn connect() -> Result<Self> {
        let stream =
            UnixStream::connect(SOCKET_PATH).context("Failed to connect to yashiki daemon")?;
        Ok(Self { stream })
    }

    pub fn send(&mut self, cmd: &Command) -> Result<Response> {
        let json = serde_json::to_string(cmd)?;
        writeln!(self.stream, "{}", json)?;
        self.stream.flush()?;

        let mut reader = BufReader::new(&self.stream);
        let mut line = String::new();
        reader.read_line(&mut line)?;

        let response: Response = serde_json::from_str(&line)?;
        Ok(response)
    }
}
