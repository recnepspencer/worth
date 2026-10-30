//! A recovery killed at each of its own stages leaves a store the next
//! recovery settles to the same journal: the interrupted publication rolls
//! forward exactly once, and nothing the first attempt did is repeated or lost.

pub use {
    std::fs, std::path::Path, std::path::PathBuf, std::process::Command, std::process::Stdio,
    std::thread, std::time::Duration, std::time::Instant,
};

pub use worth_store_recovery_runtime::WorthStoreRecovery;
pub use {
    worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage,
};

pub use {
    crate::production_entry::certified_release_serving,
    crate::production_entry::ordered_journals::canonical_journal::assert_journal_holds,
    crate::production_entry::ordered_journals::canonical_journal::child,
    crate::production_entry::ordered_journals::canonical_journal::launch,
    crate::production_entry::ordered_journals::canonical_journal::payload,
    crate::production_entry::ordered_journals::canonical_journal::placement,
    crate::production_entry::ordered_journals::canonical_journal::publication,
    crate::production_entry::ordered_journals::canonical_journal::serve,
    crate::production_entry::ordered_journals::canonical_journal::Stop,
    crate::production_entry::ordered_journals::canonical_journal::ENTRIES,
};

pub const RECOVERY_ROLE_ENV: &str = "WORTH_JOURNAL_RECOVERY_ROLE";
pub const RECOVERY_ROOT_ENV: &str = "WORTH_JOURNAL_RECOVERY_ROOT";
pub const RECOVERY_MARKERS_ENV: &str = "WORTH_JOURNAL_RECOVERY_MARKERS";
pub const RECOVERY_CHILD: &str =
    "process::recovery_runtime::production_entry::ordered_journals::canonical_journal::recovery_interruption::recovery_interruption_child";

pub const STAGES: [PhysicalRecoveryYieldpointStage; 10] = [
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::StagingMaterialization,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::StagingSynchronization,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CandidateMaterialization,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CandidateSynchronization,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::RootProtocolReplacement,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::RecordNamespaceSynchronization,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::FreshReopenCurrentSelector,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::FreshReopenExactBinding,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::CleanupRemoval,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::ObligationDischarge,
];

/// Runs recovery in a child armed at `stage`, killing it once it parks there.
/// Returns whether the stage was reached before recovery finished.
pub fn kill_recovery_at(root: &Path, stage: PhysicalRecoveryYieldpointStage) -> bool {
    let markers = tempfile::tempdir().unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", RECOVERY_CHILD, "--nocapture", "--test-threads=1"])
        .env(RECOVERY_ROLE_ENV, stage.label())
        .env(RECOVERY_ROOT_ENV, root)
        .env(RECOVERY_MARKERS_ENV, markers.path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let reached = markers.path().join("reached");
    let finished = markers.path().join("finished");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(240);
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
            std::time::Instant::now() < deadline,
            "recovery child hung at {}",
            stage.label()
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
