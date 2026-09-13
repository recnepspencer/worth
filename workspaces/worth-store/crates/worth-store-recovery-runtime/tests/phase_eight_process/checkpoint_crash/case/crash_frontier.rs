use worth_store_offline_verifier::RecoveryObserverReport;
use worth_store_recovery_runtime::RecoveryReportOutcome;

use super::super::super::history;
use super::super::evidence::{
    assert_snapshot_preserved, assert_stage_frontier, derive_expected_frontier, snapshot_directory,
};
use super::super::process::{fresh_observer, wait_for_marker};
use super::fixture_setup::CheckpointFixture;
use super::oracle::independently_classify_in_flight;

pub(super) struct CrashObservation {
    pub(super) fixture: CheckpointFixture,
    pub(super) effect_snapshot: std::collections::BTreeMap<String, (u64, [u8; 32])>,
    pub(super) expected_outcome: RecoveryReportOutcome,
    pub(super) candidate_is_residue: bool,
    pub(super) crash_observer: RecoveryObserverReport,
    pub(super) crash_history: history::ParentPhysicalHistory,
}

pub(super) fn observe(mut fixture: CheckpointFixture) -> CrashObservation {
    let stage = fixture.stage;
    let start_marker = fixture.parent.path().join("checkpoint-start");
    let reached_marker = fixture.parent.path().join("checkpoint-reached");
    std::fs::write(&start_marker, stage.label()).expect("release checkpoint writer");
    wait_for_marker(&mut fixture.child, &reached_marker, "checkpoint effect");

    let effect_snapshot = snapshot_directory(&fixture.root);
    let expected_frontier = derive_expected_frontier(stage.label());
    assert_stage_frontier(
        stage.label(),
        &fixture.root,
        &fixture.baseline_snapshot,
        &effect_snapshot,
        &expected_frontier,
    );
    let expected_outcome = expected_frontier.outcome();
    let candidate_is_residue = expected_frontier.candidate_is_residue();

    let killed = fixture
        .child
        .kill_and_wait()
        .expect("wait for killed checkpoint writer");
    assert!(!killed.success(), "writer must be killed at {stage:?}");

    let crash_observer = fresh_observer(&fixture.parent, &fixture.root, "crash-observer");
    if candidate_is_residue {
        assert!(
            crash_observer.residue_artifact_count()
                > fixture.baseline_observer.residue_artifact_count(),
            "{scenario:?} did not expose a new residue artifact after process death",
            scenario = fixture.scenario
        );
        assert!(
            crash_observer.residue_bytes() > fixture.baseline_observer.residue_bytes(),
            "{scenario:?} did not expose residue bytes after process death",
            scenario = fixture.scenario
        );
        assert_ne!(
            crash_observer.residue_digest(),
            fixture.baseline_observer.residue_digest(),
            "{scenario:?} residue digest did not change after process death",
            scenario = fixture.scenario
        );
    }
    assert_snapshot_preserved(
        &fixture.root,
        &fixture.baseline_snapshot,
        &effect_snapshot,
        stage.label(),
    );
    let crash_history =
        history::ParentPhysicalHistory::capture(&fixture.root, &fixture.operation_program.expected)
            .expect("capture parent history at killed checkpoint");
    let in_flight_fate = fixture
        .operation_program
        .expected
        .classify_in_flight_from_physical_artifacts(&fixture.root)
        .expect("classify in-flight mutation fate from selected physical evidence");
    let independent_fate =
        independently_classify_in_flight(&fixture.root, &fixture.operation_program.expected);
    assert_eq!(
        in_flight_fate, independent_fate,
        "in-flight fate reducer disagreed with raw persisted evidence at {stage:?}"
    );
    let baseline_artifacts = fixture.baseline_snapshot.len() as u64;
    assert!(crash_observer.artifact_count() >= baseline_artifacts);
    assert!(crash_observer.bytes_read() > 0);
    assert_ne!(crash_observer.artifact_set_digest(), [0; 32]);
    assert!(
        crash_history.checkpoint_count() >= 1,
        "the crash fixture must retain a checkpoint artifact at {stage:?}; observed {}",
        crash_history.checkpoint_count()
    );
    assert!(
        crash_history.latest_checkpoint_sequence() >= 2,
        "the crash fixture must cross two checkpoint intervals at {stage:?}; observed sequence {}",
        crash_history.latest_checkpoint_sequence()
    );
    assert!(
        crash_history.wal_segment_count() >= 2,
        "the crash fixture must rotate the WAL before {stage:?}; observed {} WAL segments",
        crash_history.wal_segment_count()
    );

    CrashObservation {
        fixture,
        effect_snapshot,
        expected_outcome,
        candidate_is_residue,
        crash_observer,
        crash_history,
    }
}
