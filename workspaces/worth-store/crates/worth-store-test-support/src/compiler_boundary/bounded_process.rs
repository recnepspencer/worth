use std::io::Read;
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

pub(crate) struct BoundedProcessOutput {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub timed_out: bool,
}

pub(crate) fn run(
    command: &mut Command,
    timeout: Duration,
    output_cap: usize,
) -> Result<BoundedProcessOutput, String> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    let stdout = child.stdout.take().expect("piped compiler stdout");
    let stderr = child.stderr.take().expect("piped compiler stderr");
    let stdout_reader = std::thread::spawn(move || read_stream(stdout, output_cap));
    let stderr_reader = std::thread::spawn(move || read_stream(stderr, output_cap));
    let (status, timed_out) = match wait_bounded(&mut child, timeout) {
        Ok(completion) => completion,
        Err(error) => {
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            return Err(error);
        }
    };
    let stdout = join_stream(stdout_reader, "stdout")?;
    let stderr = join_stream(stderr_reader, "stderr")?;
    Ok(BoundedProcessOutput {
        status,
        stdout,
        stderr,
        timed_out,
    })
}

fn wait_bounded(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Result<(ExitStatus, bool), String> {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok((status, false)),
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Ok(None) => {
                let kill_error = child.kill().err();
                let status = child
                    .wait()
                    .map_err(|error| format!("could not reap timed-out compiler: {error}"))?;
                if let Some(error) = kill_error {
                    return Err(format!("could not kill timed-out compiler: {error}"));
                }
                return Ok((status, true));
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("could not observe compiler process: {error}"));
            }
        }
    }
}

fn read_stream(mut stream: impl Read, output_cap: usize) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        if bytes.len().saturating_add(read) > output_cap {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("compiler output exceeded {output_cap} bytes"),
            ));
        }
        bytes.extend_from_slice(&buffer[..read]);
    }
    Ok(bytes)
}

fn join_stream(
    reader: std::thread::JoinHandle<std::io::Result<Vec<u8>>>,
    stream: &str,
) -> Result<Vec<u8>, String> {
    reader
        .join()
        .map_err(|_| format!("compiler {stream} reader panicked"))?
        .map_err(|error| format!("could not read compiler {stream}: {error}"))
}
