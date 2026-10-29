pub use std::path::Path;
pub use std::process::Stdio;
pub use std::time::Duration;

pub use serde::Deserialize;
pub use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
pub use tokio::process::{Child, ChildStdin, ChildStdout, Command};

#[derive(Deserialize)]
pub struct ProcessPosture {
    pub state: String,
    pub process_id: u32,
    pub address: String,
}

pub struct CourtroomProcess {
    pub child: Child,
    pub input: ChildStdin,
    pub output: Lines<BufReader<ChildStdout>>,
    pub local_address: Option<std::net::SocketAddr>,
}

impl CourtroomProcess {
    pub async fn spawn(executable: &Path) -> Result<(Self, ProcessPosture), String> {
        let mut child = Command::new(executable)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()
            .map_err(|error| format!("process could not start: {error}"))?;
        let input = child
            .stdin
            .take()
            .ok_or_else(|| "process stdin was unavailable".to_string())?;
        let output = child
            .stdout
            .take()
            .map(BufReader::new)
            .map(AsyncBufReadExt::lines)
            .ok_or_else(|| "process stdout was unavailable".to_string())?;
        let mut process = Self {
            child,
            input,
            output,
            local_address: None,
        };
        let posture = process.read_posture("bound").await?;
        process.local_address = Some(
            posture
                .address
                .parse()
                .map_err(|_| "process bound address was invalid".to_string())?,
        );
        Ok((process, posture))
    }

    pub fn local_address(&self) -> std::net::SocketAddr {
        self.local_address
            .expect("spawned process has reported its bound address")
    }

    pub async fn install(
        &mut self,
        configuration: serde_json::Value,
    ) -> Result<ProcessPosture, String> {
        self.write_json(&configuration).await?;
        self.read_posture("ready").await
    }

    pub async fn shutdown(mut self) -> Result<(), String> {
        self.write_json(&serde_json::json!({ "command": "shutdown" }))
            .await?;
        self.input
            .shutdown()
            .await
            .map_err(|error| format!("process stdin shutdown failed: {error}"))?;
        let status = tokio::time::timeout(Duration::from_secs(15), self.child.wait())
            .await
            .map_err(|_| "process did not terminate before deadline".to_string())?
            .map_err(|error| format!("process wait failed: {error}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("process exited unsuccessfully: {status}"))
        }
    }

    pub async fn terminate(mut self) -> Result<(), String> {
        self.child
            .kill()
            .await
            .map_err(|error| format!("process termination failed: {error}"))?;
        self.child
            .wait()
            .await
            .map_err(|error| format!("terminated process wait failed: {error}"))?;
        Ok(())
    }

    pub async fn read_posture(&mut self, expected: &str) -> Result<ProcessPosture, String> {
        let line = tokio::time::timeout(Duration::from_secs(300), self.output.next_line())
            .await
            .map_err(|_| format!("process did not report {expected} before deadline"))?
            .map_err(|error| format!("process posture read failed: {error}"))?
            .ok_or_else(|| format!("process closed before reporting {expected}"))?;
        let posture: ProcessPosture = serde_json::from_str(&line)
            .map_err(|error| format!("process posture was malformed: {error}"))?;
        if posture.state != expected {
            return Err(format!(
                "process reported `{}` while `{expected}` was required",
                posture.state
            ));
        }
        Ok(posture)
    }

    pub async fn write_json(&mut self, value: &serde_json::Value) -> Result<(), String> {
        let mut bytes = serde_json::to_vec(value)
            .map_err(|error| format!("process input encoding failed: {error}"))?;
        bytes.push(b'\n');
        self.input
            .write_all(&bytes)
            .await
            .map_err(|error| format!("process input failed: {error}"))?;
        self.input
            .flush()
            .await
            .map_err(|error| format!("process input flush failed: {error}"))
    }
}
