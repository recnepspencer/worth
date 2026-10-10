use crate::facade::{
    BridgeCanonicalHistoricalEvaluationRecord, BridgeHistoricalEvaluationReplaySummary,
    BridgeHistoricalMaterializationPath,
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

pub(super) fn assert_historical_record(
    record: &BridgeCanonicalHistoricalEvaluationRecord,
    snapshot_identity: &str,
    branch_identity: &str,
    commit_identity: &str,
    materialization_path: BridgeHistoricalMaterializationPath,
) {
    assert_eq!(
        record.decision_log().snapshot_identity().as_str(),
        crate::truth_identity_fixtures::truth_snapshot_fixture(snapshot_identity).as_str()
    );
    assert_eq!(
        record.decision_log().branch_identity().as_str(),
        crate::truth_identity_fixtures::truth_branch_fixture(branch_identity).as_str()
    );
    assert_eq!(
        record
            .decision_log()
            .commit_identity()
            .map(|identity| identity.as_str()),
        Some(crate::truth_identity_fixtures::truth_commit_fixture(commit_identity).as_str())
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
    snapshot_identity: &str,
) {
    assert_eq!(replay_summary.record_identity(), record.record_identity());
    assert_eq!(
        replay_summary.decision_log_identity(),
        record.decision_log().decision_log_identity()
    );
    assert_eq!(
        replay_summary.snapshot_identity().as_str(),
        crate::truth_identity_fixtures::truth_snapshot_fixture(snapshot_identity).as_str()
    );
}
