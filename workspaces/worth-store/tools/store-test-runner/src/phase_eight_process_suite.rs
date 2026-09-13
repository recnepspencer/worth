use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

#[path = "phase_eight_process_suite/child.rs"]
mod child;

const WRITER_ENV: &str = "WORTH_STORE_PHASE8_WRITER";
const OBSERVER_ENV: &str = "WORTH_STORE_PHASE8_OBSERVER";
const RECOVERY_ENV: &str = "WORTH_STORE_PHASE8_RECOVERY";

struct ProcessBinaries {
    target: PathBuf,
    writer: PathBuf,
    observer: PathBuf,
    recovery: PathBuf,
}

pub(super) fn run(workspace: &Path, target_root: Option<&Path>) -> Result<(), String> {
    let binaries = ProcessBinaries::build(workspace, target_root)?;
    let mut command = cargo(workspace);
    command.args([
        "test",
        "-j",
        "1",
        "-p",
        "worth-store-recovery-runtime",
        "--test",
        "phase_eight_process",
        "--features",
        "certification-test-authority",
    ]);
    command.args(["--", "--test-threads=1"]);
    command
        .env("CARGO_TARGET_DIR", &binaries.target)
        .env(WRITER_ENV, &binaries.writer)
        .env(OBSERVER_ENV, &binaries.observer)
        .env(RECOVERY_ENV, &binaries.recovery);
    successful(
        child::run_within(&mut command, Duration::from_secs(60 * 60))?,
        "Phase 8 process suite",
    )
}

impl ProcessBinaries {
    fn build(workspace: &Path, target_root: Option<&Path>) -> Result<Self, String> {
        let target = cargo_target(workspace, target_root);
        build(
            workspace,
            &target,
            "worth-store",
            "physical_store_c8_writer",
            None,
        )?;
        build(
            workspace,
            &target,
            "worth-store-offline-verifier",
            "physical_store_offline_observer",
            None,
        )?;
        build(
            workspace,
            &target,
            "worth-store-recovery-runtime",
            "physical_store_recover",
            Some("worth-store-recovery-runtime/certification-test-authority"),
        )?;
        Ok(Self {
            writer: executable(&target, "physical_store_c8_writer")?,
            observer: executable(&target, "physical_store_offline_observer")?,
            recovery: executable(&target, "physical_store_recover")?,
            target,
        })
    }
}

fn cargo_target(workspace: &Path, target_root: Option<&Path>) -> PathBuf {
    target_root
        .map(Path::to_path_buf)
        .or_else(|| std::env::var_os("CARGO_TARGET_DIR").map(PathBuf::from))
        .map(|target| {
            if target.is_absolute() {
                target
            } else {
                workspace.join(target)
            }
        })
        .unwrap_or_else(|| workspace.join("target"))
}

fn build(
    workspace: &Path,
    target: &Path,
    package: &str,
    binary: &str,
    feature: Option<&str>,
) -> Result<(), String> {
    let mut command = cargo(workspace);
    command.env("CARGO_TARGET_DIR", target).args([
        "build",
        "--locked",
        "-j",
        "1",
        "--no-default-features",
        "-p",
        package,
        "--bin",
        binary,
    ]);
    if let Some(feature) = feature {
        command.args(["--features", feature]);
    }
    successful(
        child::run_within(&mut command, Duration::from_secs(30 * 60))?,
        &format!("build {package}::{binary}"),
    )
}

fn cargo(workspace: &Path) -> Command {
    let mut command = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
    command.current_dir(workspace);
    command
}

fn executable(target: &Path, binary: &str) -> Result<PathBuf, String> {
    let path = target
        .join("debug")
        .join(format!("{binary}{}", std::env::consts::EXE_SUFFIX));
    path.is_file()
        .then_some(path)
        .ok_or_else(|| format!("Cargo did not produce `{binary}`"))
}

fn successful(status: std::process::ExitStatus, action: &str) -> Result<(), String> {
    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("{action} exited with {status}"))
}
