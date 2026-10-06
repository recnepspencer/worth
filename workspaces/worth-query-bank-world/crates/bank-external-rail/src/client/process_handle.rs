//! Spawns the external rail binary as a genuinely separate OS process and
//! discovers the TCP address it bound.
//!
//! This is the load-bearing proof for Gate 8.2: nothing here shares memory,
//! a runtime, or a truth source with the caller.

use std::io::{BufRead, BufReader, Write};
use std::net::SocketAddr;
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use crate::protocol::support_profile::RailProtocolSupportProfile;
use crate::RailCompletionDeliveryConfiguration;
use crate::RailCompletionDeliveryPosture;

/// A running, separate-process external rail.
///
/// Killing the process is this handle's responsibility: dropping it always
/// terminates the child, so a test cannot leak a rail process past its own
/// scope.
pub struct RailProcessHandle {
    child: Child,
    local_addr: SocketAddr,
    test_control_addr: SocketAddr,
    installation_input: Option<ChildStdin>,
    installation_output: Option<BufReader<ChildStdout>>,
    awaiting_installation: bool,
}

/// Failure to spawn the rail process or discover its bound address.
#[derive(Debug)]
pub enum RailSpawnError {
    Spawn(std::io::Error),
    NoStdout,
    ReadLine(std::io::Error),
    ProcessExitedBeforeListening,
    MalformedListeningLine(String),
    InstallationUnavailable,
    InstallationWrite(std::io::Error),
    InstallationEncoding(serde_json::Error),
    InstallationRejected,
    CloseUnavailable,
    CloseWrite(std::io::Error),
    CloseRead(std::io::Error),
    ClosePostureMalformed,
    CloseExit(std::io::Error),
}

impl RailProcessHandle {
    /// Spawns the rail binary at `binary_path` (for example
    /// `env!("CARGO_BIN_EXE_bank-external-rail")` from an integration test)
    /// with the given bind address (`"127.0.0.1:0"` lets the OS assign a
    /// free port) and waits for it to report the address it actually bound.
    pub fn spawn(binary_path: impl AsRef<Path>, bind_addr: &str) -> Result<Self, RailSpawnError> {
        Self::spawn_with_protocol_support(
            binary_path,
            bind_addr,
            RailProtocolSupportProfile::Current,
        )
    }

    pub fn spawn_with_protocol_support(
        binary_path: impl AsRef<Path>,
        bind_addr: &str,
        protocol_support: RailProtocolSupportProfile,
    ) -> Result<Self, RailSpawnError> {
        Self::spawn_inner(binary_path.as_ref(), bind_addr, protocol_support, false)
    }

    /// The child binds and reports its address, then waits for one stdin
    /// installation document before accepting any dispatch connections.
    pub fn spawn_awaiting_completion_installation(
        binary_path: impl AsRef<Path>,
        bind_addr: &str,
    ) -> Result<Self, RailSpawnError> {
        Self::spawn_inner(
            binary_path.as_ref(),
            bind_addr,
            RailProtocolSupportProfile::Current,
            true,
        )
    }

    fn spawn_inner(
        binary_path: &Path,
        bind_addr: &str,
        protocol_support: RailProtocolSupportProfile,
        await_installation: bool,
    ) -> Result<Self, RailSpawnError> {
        let mut command = Command::new(binary_path);
        command
            .arg(bind_addr)
            .arg(protocol_support.command_line_name());
        command
            .arg(if await_installation {
                "--completion-config-stdin"
            } else {
                "--control-stdin"
            })
            .stdin(Stdio::piped());
        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(RailSpawnError::Spawn)?;
        let installation_input = child.stdin.take();

        let stdout = child.stdout.take().ok_or(RailSpawnError::NoStdout)?;
        let mut output = BufReader::new(stdout);
        let mut listening_line = String::new();
        let listening_line = match output.read_line(&mut listening_line) {
            Ok(0) => return Err(RailSpawnError::ProcessExitedBeforeListening),
            Ok(_) => listening_line,
            Err(error) => return Err(RailSpawnError::ReadLine(error)),
        };
        let (local_addr, test_control_addr) = match parse_listening_line(listening_line.trim_end())
        {
            Some(addresses) => addresses,
            None => return Err(RailSpawnError::MalformedListeningLine(listening_line)),
        };

        Ok(Self {
            child,
            local_addr,
            test_control_addr,
            installation_input,
            installation_output: Some(output),
            awaiting_installation: await_installation,
        })
    }

    pub fn install_completion_delivery(
        &mut self,
        configuration: &RailCompletionDeliveryConfiguration,
    ) -> Result<(), RailSpawnError> {
        if !self.awaiting_installation {
            return Err(RailSpawnError::InstallationUnavailable);
        }
        let input = self
            .installation_input
            .as_mut()
            .ok_or(RailSpawnError::InstallationUnavailable)?;
        serde_json::to_writer(&mut *input, configuration)
            .map_err(RailSpawnError::InstallationEncoding)?;
        input
            .write_all(b"\n")
            .map_err(RailSpawnError::InstallationWrite)?;
        input.flush().map_err(RailSpawnError::InstallationWrite)?;
        let mut ready = String::new();
        let output = self
            .installation_output
            .as_mut()
            .ok_or(RailSpawnError::InstallationUnavailable)?;
        output
            .read_line(&mut ready)
            .map_err(RailSpawnError::ReadLine)?;
        if ready.trim_end() != "COMPLETION_DELIVERY_READY" {
            return Err(RailSpawnError::InstallationRejected);
        }
        self.awaiting_installation = false;
        Ok(())
    }

    /// Fences new rail contacts and reports unresolved sender work before the
    /// child exits. Dropping the handle remains a forced-termination fallback.
    pub fn close(mut self) -> Result<RailCompletionDeliveryPosture, RailSpawnError> {
        if self.awaiting_installation {
            return Err(RailSpawnError::CloseUnavailable);
        }
        let input = self
            .installation_input
            .as_mut()
            .ok_or(RailSpawnError::CloseUnavailable)?;
        input
            .write_all(b"CLOSE\n")
            .map_err(RailSpawnError::CloseWrite)?;
        input.flush().map_err(RailSpawnError::CloseWrite)?;
        let output = self
            .installation_output
            .as_mut()
            .ok_or(RailSpawnError::CloseUnavailable)?;
        let mut line = String::new();
        output
            .read_line(&mut line)
            .map_err(RailSpawnError::CloseRead)?;
        let posture = line
            .trim_end()
            .strip_prefix("CLOSING ")
            .and_then(|json| serde_json::from_str(json).ok())
            .ok_or(RailSpawnError::ClosePostureMalformed)?;
        let status = self.child.wait().map_err(RailSpawnError::CloseExit)?;
        if !status.success() {
            return Err(RailSpawnError::ClosePostureMalformed);
        }
        Ok(posture)
    }

    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub fn test_control_addr(&self) -> SocketAddr {
        self.test_control_addr
    }
}

impl Drop for RailProcessHandle {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn parse_listening_line(line: &str) -> Option<(SocketAddr, SocketAddr)> {
    let addresses = line.strip_prefix("LISTENING ")?;
    let (production, test_control) = addresses.split_once(" TEST_CONTROL ")?;
    Some((production.parse().ok()?, test_control.parse().ok()?))
}
