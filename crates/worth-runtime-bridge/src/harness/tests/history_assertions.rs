use crate::facade::{
    BridgeCanonicalHistoricalEvaluationRecord, BridgeHistoricalEvaluationReplaySummary,
    BridgeHistoricalMaterializationPath, TruthBranchIdentity, TruthCommitIdentity,
    TruthSnapshotIdentity,
};
use crate::harness::adapter::BridgeHarnessSession;

pub(super) fn last_historical_record(
    session: &BridgeHarnessSession,
) -> BridgeCanonicalHistoricalEvaluationRecord {
    session
        .runtime
        .as_ref()
        .expect("bridge runtime")
        .diagnostics()
        .last_historical_evaluation_record()
        .expect("historical evaluation record should be retained")
}

pub(super) fn replay_historical_record(
    session: &BridgeHarnessSession,
    record: &BridgeCanonicalHistoricalEvaluationRecord,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> BridgeHistoricalEvaluationReplaySummary {
    session
        .runtime
        .as_ref()
        .expect("bridge runtime")
        .replay_canonical_historical_evaluation_record(record, execution)
        .expect("historical replay should reconstruct the canonical record")
}

pub(super) fn assert_native_historical_record(
    record: &BridgeCanonicalHistoricalEvaluationRecord,
    snapshot_identity: &TruthSnapshotIdentity,
    branch_identity: &TruthBranchIdentity,
    commit_identity: &TruthCommitIdentity,
    materialization_path: BridgeHistoricalMaterializationPath,
) {
    assert_eq!(record.decision_log().snapshot_identity(), snapshot_identity);
    assert_eq!(record.decision_log().branch_identity(), branch_identity);
    assert_eq!(
        record.decision_log().commit_identity(),
        Some(commit_identity)
    );
    assert_eq!(
        record.decision_log().materialization_path(),
        materialization_path
    );
    assert_eq!(record.counters().truth_view_decision_log_count(), 1);
}

pub(super) fn assert_historical_replay_summary(
    replay_summary: &BridgeHistoricalEvaluationReplaySummary,
    record: &BridgeCanonicalHistoricalEvaluationRecord,
    snapshot_identity: &TruthSnapshotIdentity,
) {
    assert_eq!(replay_summary.record_identity(), record.record_identity());
    assert_eq!(
        replay_summary.decision_log_identity(),
        record.decision_log().decision_log_identity()
    );
    assert_eq!(replay_summary.snapshot_identity(), snapshot_identity);
}
