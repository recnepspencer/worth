//! A recovery killed at each of its own stages leaves a store the next
//! recovery settles to the same journal: the interrupted publication rolls
//! forward exactly once, and nothing the first attempt did is repeated or lost.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use worth_store::physical_runtime::{
    PhysicalRecoveryProcessYieldpoint, PhysicalRecoveryYieldpointStage,
};
use worth_store_recovery_runtime::WorthStoreRecovery;

use super::*;

const RECOVERY_ROLE_ENV: &str = "WORTH_JOURNAL_RECOVERY_ROLE";
const RECOVERY_ROOT_ENV: &str = "WORTH_JOURNAL_RECOVERY_ROOT";
const RECOVERY_MARKERS_ENV: &str = "WORTH_JOURNAL_RECOVERY_MARKERS";
const RECOVERY_CHILD: &str =
    "canonical_journal::recovery_interruption::recovery_interruption_child";

const STAGES: [PhysicalRecoveryYieldpointStage; 9] = [
    PhysicalRecoveryYieldpointStage::StagingMaterialization,
    PhysicalRecoveryYieldpointStage::StagingSynchronization,
    PhysicalRecoveryYieldpointStage::CandidateMaterialization,
    PhysicalRecoveryYieldpointStage::CandidateSynchronization,
    PhysicalRecoveryYieldpointStage::RootProtocolReplacement,
    PhysicalRecoveryYieldpointStage::RecordNamespaceSynchronization,
    PhysicalRecoveryYieldpointStage::FreshReopenCurrentSelector,
    PhysicalRecoveryYieldpointStage::FreshReopenExactBinding,
    PhysicalRecoveryYieldpointStage::CleanupRemoval,
];

/// Runs recovery in a child armed at `stage`, killing it once it parks there.
/// Returns whether the stage was reached before recovery finished.
fn kill_recovery_at(root: &Path, stage: PhysicalRecoveryYieldpointStage) -> bool {
    let markers = tempfile::tempdir().unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", RECOVERY_CHILD, "--nocapture", "--test-threads=1"])
        .env(RECOVERY_ROLE_ENV, stage.label())
        .env(RECOVERY_ROOT_ENV, root)
        .env(RECOVERY_MARKERS_ENV, markers.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let reached = markers.path().join("reached");
    let finished = markers.path().join("finished");
    let deadline = Instant::now() + Duration::from_secs(240);
    loop {
        if reached.is_file() {
            child.kill().unwrap();
            child.wait().unwrap();
            return true;
        }
        if finished.is_file() || child.try_wait().unwrap().is_some() {
            let output = child.wait_with_output().unwrap();
            assert!(
                finished.is_file(),
                "recovery child failed at {}: {}",
                stage.label(),
                String::from_utf8_lossy(&output.stderr)
            );
            return false;
        }
        assert!(
            Instant::now() < deadline,
            "recovery child hung at {}",
            stage.label()
        );
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn a_recovery_killed_at_each_of_its_stages_settles_to_the_same_journal() {
    let mut reached_any = false;
    for stage in STAGES {
        let (root, _marker) = launch(Stop::AfterWalDurability);
        reached_any |= kill_recovery_at(&root, stage);
        let serving = serve(&root, stage.label());
        assert_journal_holds(&serving, ENTRIES);
        publication(&serving, placement(), ENTRIES + 1, payload(ENTRIES + 1))
            .expect("publication after an interrupted recovery");
        serving.close();
        let serving = serve(&root, "recovery after the interrupted one");
        assert_journal_holds(&serving, ENTRIES + 1);
        serving.close();
    }
    assert!(reached_any, "no recovery stage was interrupted");
}

#[test]
fn recovery_interruption_child() {
    let Some(label) = std::env::var_os(RECOVERY_ROLE_ENV) else {
        return;
    };
    let stage = PhysicalRecoveryYieldpointStage::from_label(label.to_str().unwrap())
        .expect("recovery stage label");
    let root = PathBuf::from(std::env::var_os(RECOVERY_ROOT_ENV).unwrap());
    let markers = PathBuf::from(std::env::var_os(RECOVERY_MARKERS_ENV).unwrap());
    let yieldpoint = PhysicalRecoveryProcessYieldpoint::new(
        stage,
        markers.join("reached"),
        markers.join("release"),
        markers.join("cancel"),
    );
    let _ = WorthStoreRecovery::recover_with_process_yieldpoint(
        certified_release_serving::request(&root),
        yieldpoint,
    );
    fs::write(markers.join("finished"), b"1").unwrap();
}
