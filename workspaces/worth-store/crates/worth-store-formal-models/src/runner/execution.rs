use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};

use super::{
    interpret_tlc_output, ProtocolCheckInvocation, ProtocolCheckVerdict, PINNED_TLC_SHA256,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlcRunnerPaths {
    java_executable: PathBuf,
    tool_jar: PathBuf,
    state_directory: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolRunnerFailure {
    MissingJavaExecutable,
    MissingToolJar,
    ToolDigestMismatch {
        actual: String,
    },
    MissingModel,
    MissingConfiguration,
    StateDirectoryNotFresh,
    ProcessLaunch(String),
    ProcessTimedOut,
    CheckerOutput {
        denial: super::ProtocolCheckerOutputDenial,
        output: String,
    },
}

impl TlcRunnerPaths {
    pub fn new(
        java_executable: impl Into<PathBuf>,
        tool_jar: impl Into<PathBuf>,
        state_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            java_executable: java_executable.into(),
            tool_jar: tool_jar.into(),
            state_directory: state_directory.into(),
        }
    }
}

pub fn execute_protocol_check(
    invocation: &ProtocolCheckInvocation,
    runner: &TlcRunnerPaths,
) -> Result<ProtocolCheckVerdict, ProtocolRunnerFailure> {
    require_file(
        &runner.java_executable,
        ProtocolRunnerFailure::MissingJavaExecutable,
    )?;
    require_file(&runner.tool_jar, ProtocolRunnerFailure::MissingToolJar)?;
    require_pinned_tool_digest(&runner.tool_jar)?;
    require_file(invocation.model_path(), ProtocolRunnerFailure::MissingModel)?;
    require_file(
        invocation.configuration_path(),
        ProtocolRunnerFailure::MissingConfiguration,
    )?;
    let java_executable = canonical_file(&runner.java_executable)?;
    let tool_jar = canonical_file(&runner.tool_jar)?;
    prepare_state_directory(&runner.state_directory)?;

    let mut child = Command::new(java_executable)
        .current_dir(
            invocation
                .model_path()
                .parent()
                .ok_or(ProtocolRunnerFailure::MissingModel)?,
        )
        .args(["-cp"])
        .arg(tool_jar)
        .args(["tlc2.TLC", "-deadlock", "-workers", "auto", "-metadir"])
        .arg(&runner.state_directory)
        .arg("-config")
        .arg(invocation.configuration_path())
        .arg(invocation.model_path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| ProtocolRunnerFailure::ProcessLaunch(error.to_string()))?;
    let stdout = child.stdout.take().expect("piped checker stdout");
    let stderr = child.stderr.take().expect("piped checker stderr");
    let stdout_reader = std::thread::spawn(move || read_process_stream(stdout));
    let stderr_reader = std::thread::spawn(move || read_process_stream(stderr));
    let deadline =
        Instant::now() + Duration::from_millis(invocation.bounds().maximum_runtime_millis().get());
    let status = match wait_for_checker(&mut child, deadline) {
        Ok(status) => status,
        Err(failure) => {
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            return Err(failure);
        }
    };
    let stdout = stdout_reader.join().map_err(|_| {
        ProtocolRunnerFailure::ProcessLaunch("checker stdout reader panicked".into())
    })??;
    let stderr = stderr_reader.join().map_err(|_| {
        ProtocolRunnerFailure::ProcessLaunch("checker stderr reader panicked".into())
    })??;
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&stdout),
        String::from_utf8_lossy(&stderr)
    );
    interpret_tlc_output(
        invocation.protocol(),
        invocation.bounds(),
        &combined,
        status.success(),
    )
    .map_err(|denial| ProtocolRunnerFailure::CheckerOutput {
        denial,
        output: combined,
    })
}

fn canonical_file(path: &Path) -> Result<PathBuf, ProtocolRunnerFailure> {
    std::fs::canonicalize(path)
        .map_err(|error| ProtocolRunnerFailure::ProcessLaunch(error.to_string()))
}

fn prepare_state_directory(path: &Path) -> Result<(), ProtocolRunnerFailure> {
    let parent = path
        .parent()
        .ok_or(ProtocolRunnerFailure::StateDirectoryNotFresh)?;
    std::fs::create_dir_all(parent)
        .map_err(|error| ProtocolRunnerFailure::ProcessLaunch(error.to_string()))?;
    std::fs::create_dir(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            ProtocolRunnerFailure::StateDirectoryNotFresh
        } else {
            ProtocolRunnerFailure::ProcessLaunch(error.to_string())
        }
    })
}

fn wait_for_checker(
    child: &mut Child,
    deadline: Instant,
) -> Result<ExitStatus, ProtocolRunnerFailure> {
    loop {
        let status = match child.try_wait() {
            Ok(status) => status,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(ProtocolRunnerFailure::ProcessLaunch(error.to_string()));
            }
        };
        if let Some(status) = status {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            let kill_error = child.kill().err();
            child
                .wait()
                .map_err(|error| ProtocolRunnerFailure::ProcessLaunch(error.to_string()))?;
            if let Some(error) = kill_error {
                return Err(ProtocolRunnerFailure::ProcessLaunch(format!(
                    "could not kill timed-out checker: {error}"
                )));
            }
            return Err(ProtocolRunnerFailure::ProcessTimedOut);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn read_process_stream(mut stream: impl Read) -> Result<Vec<u8>, ProtocolRunnerFailure> {
    const MAX_CHECKER_OUTPUT_BYTES: usize = 64 * 1024 * 1024;
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = stream
            .read(&mut buffer)
            .map_err(|error| ProtocolRunnerFailure::ProcessLaunch(error.to_string()))?;
        if read == 0 {
            break;
        }
        if bytes.len().saturating_add(read) > MAX_CHECKER_OUTPUT_BYTES {
            return Err(ProtocolRunnerFailure::ProcessLaunch(
                "checker output exceeded 64 MiB".to_owned(),
            ));
        }
        bytes.extend_from_slice(&buffer[..read]);
    }
    Ok(bytes)
}

fn require_pinned_tool_digest(path: &Path) -> Result<(), ProtocolRunnerFailure> {
    let digest = file_digest(path)?;
    let actual = hex_digest(&digest);
    if actual == PINNED_TLC_SHA256 {
        Ok(())
    } else {
        Err(ProtocolRunnerFailure::ToolDigestMismatch { actual })
    }
}

fn file_digest(path: &Path) -> Result<[u8; 32], ProtocolRunnerFailure> {
    let mut file = std::fs::File::open(path)
        .map_err(|error| ProtocolRunnerFailure::ProcessLaunch(error.to_string()))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| ProtocolRunnerFailure::ProcessLaunch(error.to_string()))?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(digest.finalize().into())
}

fn hex_digest(digest: &[u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn require_file(path: &Path, denial: ProtocolRunnerFailure) -> Result<(), ProtocolRunnerFailure> {
    if path.is_file() {
        Ok(())
    } else {
        Err(denial)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checker_process_is_killed_when_its_runtime_bound_expires() {
        let mut child = sleeping_child();
        let result = wait_for_checker(&mut child, Instant::now());

        assert_eq!(result, Err(ProtocolRunnerFailure::ProcessTimedOut));
        assert!(child.try_wait().unwrap().is_some());
    }

    #[cfg(windows)]
    fn sleeping_child() -> Child {
        Command::new("cmd")
            .args(["/C", "ping -n 30 127.0.0.1 >NUL"])
            .spawn()
            .unwrap()
    }

    #[cfg(unix)]
    fn sleeping_child() -> Child {
        Command::new("sh").args(["-c", "sleep 30"]).spawn().unwrap()
    }
}
