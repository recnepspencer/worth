use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use super::execution_mode;
use super::process_execution::{self, DEADLINE_ENV};
use super::report_join;
use super::shard_admission;
use super::shard_contract;

const EVIDENCE_PREFIX: &str = "WORTH_UI_PHASE5_PRODUCTION_LOCALITY=";

#[expect(
    clippy::disallowed_methods,
    reason = "certification-harness thread-spawn=1: Certification drives independent actors to expose concurrency behavior."
)]
pub(super) fn execute(
    worker_executable: &Path,
    worker_arguments: &[&str],
) -> Result<Vec<serde_json::Value>, String> {
    let deadline = process_execution::new_deadline()?;
    let reports = LocalReportDirectory::new()?;
    let cancellation = Arc::new(AtomicBool::new(false));
    let executable = worker_executable.to_path_buf();
    let worker_arguments = worker_arguments
        .iter()
        .map(|argument| (*argument).to_owned())
        .collect::<Vec<_>>();
    let mut pending = (0..shard_contract::SHARD_COUNT).collect::<Vec<_>>();

    while !pending.is_empty() {
        let wave = shard_admission::next_wave(&pending);
        shard_admission::validate_wave(&wave)?;
        let workers = wave
            .iter()
            .copied()
            .map(|shard| {
                let executable = executable.clone();
                let worker_arguments = worker_arguments.clone();
                let report_path = reports.path.clone();
                let cancel_file = reports.cancel_file.clone();
                let cancellation = Arc::clone(&cancellation);
                std::thread::spawn(move || {
                    let result = execute_shard_process(
                        &executable,
                        &worker_arguments,
                        &report_path,
                        &cancel_file,
                        shard,
                        deadline,
                        &cancellation,
                    );
                    if result.is_err() {
                        cancellation.store(true, Ordering::Release);
                        let _ = std::fs::write(&cancel_file, b"cancelled");
                    }
                    result
                })
            })
            .collect::<Vec<_>>();
        let mut failure = None;
        for worker in workers {
            match worker.join() {
                Ok(Ok(())) => {}
                Ok(Err(denial)) if failure.is_none() => failure = Some(denial),
                Ok(Err(_)) => {}
                Err(_) if failure.is_none() => {
                    failure = Some("matrix shard worker panicked".to_owned())
                }
                Err(_) => {}
            }
        }
        if let Some(denial) = failure {
            return Err(denial);
        }
        pending.retain(|shard| !wave.contains(shard));
    }

    report_join::join(&reports.path)
}

fn execute_shard_process(
    executable: &Path,
    worker_arguments: &[String],
    reports: &Path,
    cancel_file: &Path,
    shard: usize,
    deadline: u128,
    cancellation: &AtomicBool,
) -> Result<(), String> {
    let label = format!("shard-{shard}-of-{}", shard_contract::SHARD_COUNT);
    let mut command = Command::new(executable);
    command
        .args(worker_arguments)
        .env(execution_mode::MODE_ENV, execution_mode::SHARD_MODE)
        .env(
            execution_mode::SHARD_ENV,
            format!("{shard}/{}", shard_contract::SHARD_COUNT),
        )
        .env(DEADLINE_ENV, deadline.to_string())
        .env(process_execution::CANCEL_ENV, cancel_file)
        .env_remove(execution_mode::CASE_ENV)
        .env_remove(execution_mode::JOIN_ENV);
    let output = process_execution::run_until_with_cancellation(
        &mut command,
        deadline,
        &label,
        Some(cancellation),
    )?;
    if !output.status().success() {
        return Err(format!(
            "matrix {label} exited {:?}",
            output.status().code()
        ));
    }
    let rows = parse_shard_rows(output.stdout(), &label)?;
    if rows.len() != shard_contract::expected_rows(shard) {
        return Err(format!(
            "matrix {label} emitted {} rows instead of {}",
            rows.len(),
            shard_contract::expected_rows(shard)
        ));
    }
    write_report(reports, shard, output.stdout())
}

fn parse_shard_rows(bytes: &[u8], label: &str) -> Result<Vec<serde_json::Value>, String> {
    let stdout = std::str::from_utf8(bytes)
        .map_err(|denial| format!("matrix {label} output encoding: {denial}"))?;
    let payloads = stdout
        .lines()
        .filter_map(|line| line.strip_prefix(EVIDENCE_PREFIX))
        .collect::<Vec<_>>();
    let [payload] = payloads.as_slice() else {
        return Err(format!(
            "matrix {label} emitted {} evidence payloads",
            payloads.len()
        ));
    };
    serde_json::from_str(payload)
        .map_err(|denial| format!("matrix {label} evidence encoding: {denial}"))
}

fn write_report(directory: &Path, shard: usize, bytes: &[u8]) -> Result<(), String> {
    let report = directory.join(shard_contract::report_name(shard));
    let temporary = directory.join(format!(
        ".{}.tmp",
        report.file_name().unwrap().to_string_lossy()
    ));
    std::fs::write(&temporary, bytes)
        .map_err(|denial| format!("matrix shard {shard} report write: {denial}"))?;
    std::fs::rename(&temporary, &report)
        .map_err(|denial| format!("matrix shard {shard} report publish: {denial}"))
}

struct LocalReportDirectory {
    path: PathBuf,
    cancel_file: PathBuf,
}

impl LocalReportDirectory {
    fn new() -> Result<Self, String> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|denial| format!("matrix report clock: {denial}"))?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "worth-ui-phase5-locality-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&path)
            .map_err(|denial| format!("matrix report directory: {denial}"))?;
        Ok(Self {
            cancel_file: path.join("cancelled"),
            path,
        })
    }
}

impl Drop for LocalReportDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::write_report;

    #[test]
    fn report_write_is_published_as_one_named_file() {
        let directory = std::env::temp_dir().join(format!(
            "worth-ui-phase5-report-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        write_report(&directory, 0, b"WORTH_UI_PHASE5_PRODUCTION_LOCALITY=[]\n").unwrap();
        assert!(directory.join("worth-ui-phase5-locality-0.jsonl").is_file());
        let _ = std::fs::remove_dir_all(directory);
    }
}
