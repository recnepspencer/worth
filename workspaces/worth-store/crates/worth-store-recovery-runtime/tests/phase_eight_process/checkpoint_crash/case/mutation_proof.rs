use worth_store_recovery_runtime::{
    RecoveryReportBlockCause, RecoveryReportDenialCause, RecoveryReportOutcome,
};

use super::super::evidence::copy_directory;
use super::super::process::{fresh_observer_raw, fresh_recovery_raw};
use super::crash_frontier::CrashObservation;
use super::mutation::{mutate_persisted_wal_frame, PERSISTED_WAL_FRAME_BYTE_FLIP};

pub(super) fn prove(crash: &CrashObservation) {
    if crash.fixture.scenario_index != 0 {
        return;
    }
    let parent = &crash.fixture.parent;
    let root = &crash.fixture.root;
    let mutated_root = parent.path().join("persisted-wal-byte-flip-root");
    copy_directory(root, &mutated_root);
    mutate_persisted_wal_frame(&mutated_root);

    let (observer_output, observer_report) =
        fresh_observer_raw(parent, &mutated_root, "persisted-wal-byte-flip-observer");
    assert!(
        !observer_output.status.success(),
        "{PERSISTED_WAL_FRAME_BYTE_FLIP} observer must reject the mutated persisted root: report={observer_report:?}; stdout={}; stderr={}",
        String::from_utf8_lossy(&observer_output.stdout),
        String::from_utf8_lossy(&observer_output.stderr)
    );
    assert!(
        observer_report.is_none(),
        "{PERSISTED_WAL_FRAME_BYTE_FLIP} observer must not emit a report for corrupted bytes"
    );
    assert!(
        String::from_utf8_lossy(&observer_output.stderr)
            .contains("physical_store_offline_observer:"),
        "{PERSISTED_WAL_FRAME_BYTE_FLIP} observer denial must be surfaced by the fresh process"
    );

    let (recovery_output, recovery_report) =
        fresh_recovery_raw(parent, &mutated_root, "persisted-wal-byte-flip-recovery");
    let recovery_report = recovery_report
        .expect("{PERSISTED_WAL_FRAME_BYTE_FLIP} recovery must emit a typed denial report");
    assert!(
        !recovery_output.status.success(),
        "{PERSISTED_WAL_FRAME_BYTE_FLIP} recovery must reject the mutated persisted root"
    );
    assert_eq!(recovery_report.outcome(), RecoveryReportOutcome::Blocked);
    assert_eq!(
        recovery_report.denial_cause(),
        Some(RecoveryReportDenialCause::Blocked(
            RecoveryReportBlockCause::WalInventory,
        ))
    );
    assert!(
        String::from_utf8_lossy(&recovery_output.stderr).contains("C8_RECOVERY_BLOCKED"),
        "{PERSISTED_WAL_FRAME_BYTE_FLIP} typed recovery denial must be rendered"
    );
    std::fs::remove_dir_all(&mutated_root).expect("remove persisted WAL mutation root");
}
