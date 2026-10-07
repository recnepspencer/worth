use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crate::plan::FocusSelection;
use crate::product::FocusGroup;

#[path = "phase_eight_process_suite/child.rs"]
mod child;
mod profile;
pub(crate) use profile::ProcessSuiteProfile;

const WRITER_ENV: &str = "WORTH_STORE_PHASE8_WRITER";
const OBSERVER_ENV: &str = "WORTH_STORE_PHASE8_OBSERVER";
const RECOVERY_ENV: &str = "WORTH_STORE_PHASE8_RECOVERY";
// The children are killed, never power-cut, so they skip the OS flush exactly
// as the in-process suites do.
const WRITER_FEATURES: &str = "volatile-sync-for-tests";
const RECOVERY_FEATURES: &str =
    "worth-store-recovery-runtime/certification-test-authority,worth-store/volatile-sync-for-tests";

struct ProcessBinaries {
    target: PathBuf,
    writer: PathBuf,
    observer: PathBuf,
    recovery: PathBuf,
}

const PROCESS_GROUPS: &[FocusGroup] = &[
    FocusGroup::PhaseOracle,
    FocusGroup::PhaseCheckpoint,
    FocusGroup::PhaseAgreement,
    FocusGroup::PhaseRecovery,
    FocusGroup::PhaseMutation,
    FocusGroup::PhaseSuccessor,
];

pub(super) fn run(
    workspace: &Path,
    target_root: Option<&Path>,
    group: Option<FocusGroup>,
    profile: ProcessSuiteProfile,
) -> Result<(), String> {
    let groups = match group.as_ref() {
        Some(group) if PROCESS_GROUPS.contains(group) => std::slice::from_ref(group),
        Some(group) => return Err(format!("{} is not a Phase 8 process group", group.as_str())),
        None => PROCESS_GROUPS,
    };
    let binaries = ProcessBinaries::build(workspace, target_root, profile)?;
    for group in groups {
        let mut command = cargo(workspace);
        command
            .args(profile.nextest_arguments(FocusSelection::for_group(*group).nextest_arguments()));
        command
            .arg("--target-dir")
            .arg(&binaries.target)
            .env(WRITER_ENV, &binaries.writer)
            .env(OBSERVER_ENV, &binaries.observer)
            .env(RECOVERY_ENV, &binaries.recovery);
        successful(
            child::run_within(&mut command, Duration::from_secs(60 * 60))?,
            group.as_str(),
        )?;
    }
    Ok(())
}

impl ProcessBinaries {
    fn build(
        workspace: &Path,
        target_root: Option<&Path>,
        profile: ProcessSuiteProfile,
    ) -> Result<Self, String> {
        let target = cargo_target(workspace, target_root);
        build(
            workspace,
            &target,
            profile,
            "worth-store",
            "physical_store_c8_writer",
            Some(WRITER_FEATURES),
        )?;
        build(
            workspace,
            &target,
            profile,
            "worth-store-offline-verifier",
            "physical_store_offline_observer",
            None,
        )?;
        build(
            workspace,
            &target,
            profile,
            "worth-store-recovery-runtime",
            "physical_store_recover",
            Some(RECOVERY_FEATURES),
        )?;
        Ok(Self {
            writer: executable(&target, profile, "physical_store_c8_writer")?,
            observer: executable(&target, profile, "physical_store_offline_observer")?,
            recovery: executable(&target, profile, "physical_store_recover")?,
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
    profile: ProcessSuiteProfile,
    package: &str,
    binary: &str,
    feature: Option<&str>,
) -> Result<(), String> {
    let mut command = cargo(workspace);
    command.arg("build").arg("--target-dir").arg(target).args([
        "--manifest-path",
        "Cargo.toml",
        "--locked",
        "--no-default-features",
        "--profile",
        profile.cargo_profile(),
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

fn executable(
    target: &Path,
    profile: ProcessSuiteProfile,
    binary: &str,
) -> Result<PathBuf, String> {
    let path = target
        .join(profile.directory())
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
