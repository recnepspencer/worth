use super::*;
use crate::truth_identity_fixtures::{truth_branch, truth_commit, truth_snapshot};

#[test]
fn runtime_admits_snapshot_bound_truth_view_policy() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::branch_snapshot(truth_branch("analysis"), truth_snapshot(1, 1)),
        BridgeReplayMode::Enabled,
        BridgeDiagnosticsTier::Standard,
        BridgeDeliveryIntent::PrepareSignalEvaluation,
    );

    let resolution = runtime.resolve_truth_view_policy(&declaration);
    match resolution {
        BridgeTruthViewPolicyResolution::Admitted(policy) => {
            assert_eq!(
                policy.retention_admission(),
                crate::snapshot::TruthViewRetentionAdmission::SnapshotResident
            );
            assert_eq!(
                policy.source_capability(),
                crate::snapshot::TruthViewSourceCapability::DirectSnapshotRead
            );
        }
        BridgeTruthViewPolicyResolution::Rejected(rejection) => {
            panic!(
                "expected admitted policy, got rejection: {}",
                rejection.detail()
            )
        }
    }
}

#[test]
fn runtime_rejects_required_replay_when_runtime_policy_disallows_replay_artifacts() {
    let runtime = runtime(
        BridgeRuntimePolicy::operational()
            .with_route_record_limit(8)
            .with_replay_artifacts(false),
    );
    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::historical_commit(truth_branch("main"), truth_commit(1)),
        BridgeReplayMode::Required,
        BridgeDiagnosticsTier::Exhaustive,
        BridgeDeliveryIntent::PrepareOnly,
    );

    let resolution = runtime.resolve_truth_view_policy(&declaration);
    match resolution {
        BridgeTruthViewPolicyResolution::Rejected(rejection) => {
            assert_eq!(
                rejection.kind(),
                crate::snapshot::TruthViewPolicyRejectionKind::ReplayNotPermitted
            );
        }
        BridgeTruthViewPolicyResolution::Admitted(_) => {
            panic!("expected replay policy rejection")
        }
    }
}

#[test]
fn runtime_plans_truth_view_packet_from_admitted_policy() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::branch_snapshot(truth_branch("analysis"), truth_snapshot(1, 1)),
        BridgeReplayMode::Enabled,
        BridgeDiagnosticsTier::Standard,
        BridgeDeliveryIntent::PrepareSignalEvaluation,
    );

    let planned = runtime
        .plan_truth_view_packet(declaration.clone(), SnapshotReadPacket::new(vec![]))
        .expect("snapshot-bound declaration should plan");

    assert_eq!(
        planned.declaration().declaration_identity(),
        declaration.declaration_identity()
    );
    assert_eq!(
        planned
            .authority_basis()
            .snapshot_identity()
            .and_then(TruthSnapshotIdentity::relational_snapshot_parts),
        Some(crate::facade::RelationalBridgeSnapshotIdentityParts::new(
            1, 1
        ))
    );
}

#[test]
fn runtime_admits_commit_bound_truth_view_policy() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::historical_commit(truth_branch("analysis"), truth_commit(1)),
        BridgeReplayMode::Enabled,
        BridgeDiagnosticsTier::Standard,
        BridgeDeliveryIntent::PrepareSignalEvaluation,
    );

    let resolution = runtime.resolve_truth_view_policy(&declaration);
    match resolution {
        BridgeTruthViewPolicyResolution::Admitted(policy) => {
            assert_eq!(
                policy.retention_admission(),
                crate::snapshot::TruthViewRetentionAdmission::HistoricalLookupRequired
            );
        }
        BridgeTruthViewPolicyResolution::Rejected(rejection) => {
            panic!(
                "expected commit-bound selector admission, got rejection: {}",
                rejection.detail()
            )
        }
    }
}

#[test]
fn runtime_materializes_commit_bound_truth_view_observation() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::historical_commit(truth_branch("analysis"), truth_commit(1)),
        BridgeReplayMode::Enabled,
        BridgeDiagnosticsTier::Standard,
        BridgeDeliveryIntent::PrepareSignalEvaluation,
    );
    let planned = runtime
        .plan_truth_view_packet(declaration, SnapshotReadPacket::new(vec![]))
        .expect("commit-bound declaration should plan");

    let observation = runtime
        .materialize_truth_view_observation(planned)
        .expect("commit-bound declaration should materialize");

    assert_eq!(
        observation.snapshot_identity().relational_snapshot_parts(),
        Some(crate::facade::RelationalBridgeSnapshotIdentityParts::new(
            1, 1
        ))
    );
    assert_eq!(
        observation
            .authority_basis()
            .commit_identity()
            .and_then(crate::input::envelope::TruthCommitIdentity::relational_commit_id),
        Some(1)
    );
}

#[test]
fn runtime_materializes_branch_head_truth_view_observation() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::branch_head(truth_branch("analysis")),
        BridgeReplayMode::Enabled,
        BridgeDiagnosticsTier::Standard,
        BridgeDeliveryIntent::PrepareSignalEvaluation,
    );
    let planned = runtime
        .plan_truth_view_packet(declaration, SnapshotReadPacket::new(vec![]))
        .expect("branch-head declaration should plan");

    let observation = runtime
        .materialize_truth_view_observation(planned)
        .expect("branch-head declaration should materialize");

    assert_eq!(
        observation.snapshot_identity().relational_snapshot_parts(),
        Some(crate::facade::RelationalBridgeSnapshotIdentityParts::new(
            1, 1
        ))
    );
    assert_eq!(
        observation
            .authority_basis()
            .commit_identity()
            .and_then(crate::input::envelope::TruthCommitIdentity::relational_commit_id),
        Some(100)
    );
}

#[test]
fn runtime_materializes_snapshot_bound_truth_view_observation() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::branch_snapshot(truth_branch("analysis"), truth_snapshot(1, 1)),
        BridgeReplayMode::Enabled,
        BridgeDiagnosticsTier::Standard,
        BridgeDeliveryIntent::PrepareSignalEvaluation,
    );
    let planned = runtime
        .plan_truth_view_packet(declaration, SnapshotReadPacket::new(vec![]))
        .expect("snapshot-bound declaration should plan");

    let observation = runtime
        .materialize_truth_view_observation(planned)
        .expect("snapshot-bound declaration should materialize");
    let validated_reads = observation
        .read_planned_packet(worth_execution::ExecutionRequest::serial(
            &crate::snapshot::test_serial_request(),
        ))
        .expect("materialized observation should execute its planned packet");

    assert_eq!(
        observation.snapshot_identity().relational_snapshot_parts(),
        Some(crate::facade::RelationalBridgeSnapshotIdentityParts::new(
            1, 1
        ))
    );
    assert_eq!(
        observation
            .snapshot_token()
            .snapshot_identity()
            .relational_snapshot_parts(),
        Some(crate::facade::RelationalBridgeSnapshotIdentityParts::new(
            1, 1
        ))
    );
    assert_eq!(
        validated_reads
            .snapshot_identity()
            .relational_snapshot_parts(),
        Some(crate::facade::RelationalBridgeSnapshotIdentityParts::new(
            1, 1
        ))
    );
}

#[test]
fn runtime_canonicalizes_historical_evaluation_record() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::historical_commit(truth_branch("analysis"), truth_commit(1)),
        BridgeReplayMode::Enabled,
        BridgeDiagnosticsTier::Standard,
        BridgeDeliveryIntent::PrepareSignalEvaluation,
    );
    let planned = runtime
        .plan_truth_view_packet(declaration, SnapshotReadPacket::new(vec![]))
        .expect("historical declaration should plan");
    let observation = runtime
        .materialize_truth_view_observation(planned)
        .expect("historical declaration should materialize");

    let record = runtime.canonicalize_historical_evaluation_record(&observation);

    assert_eq!(
        record
            .decision_log()
            .snapshot_identity()
            .relational_snapshot_parts(),
        Some(crate::facade::RelationalBridgeSnapshotIdentityParts::new(
            1, 1
        ))
    );
    assert_eq!(
        record.decision_log().materialization_path(),
        BridgeHistoricalMaterializationPath::CommitEnvelopeSnapshot
    );
    assert_eq!(
        runtime
            .diagnostics()
            .last_historical_evaluation_record()
            .expect("historical record should be retained")
            .record_identity(),
        record.record_identity()
    );
}

#[test]
fn runtime_lowers_identical_historical_requests_to_identical_artifacts() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::historical_commit(truth_branch("analysis"), truth_commit(1)),
        BridgeReplayMode::Enabled,
        BridgeDiagnosticsTier::Standard,
        BridgeDeliveryIntent::PrepareSignalEvaluation,
    );
    let left_observation = runtime
        .materialize_truth_view_observation(
            runtime
                .plan_truth_view_packet(declaration.clone(), SnapshotReadPacket::new(vec![]))
                .expect("left historical declaration should plan"),
        )
        .expect("left historical declaration should materialize");
    let right_observation = runtime
        .materialize_truth_view_observation(
            runtime
                .plan_truth_view_packet(declaration, SnapshotReadPacket::new(vec![]))
                .expect("right historical declaration should plan"),
        )
        .expect("right historical declaration should materialize");

    let left = runtime.lower_historical_evaluation_artifact(&left_observation);
    let right = runtime.lower_historical_evaluation_artifact(&right_observation);

    assert_eq!(left, right);
    assert_eq!(
        left.snapshot_identity().relational_snapshot_parts(),
        Some(crate::facade::RelationalBridgeSnapshotIdentityParts::new(
            1, 1
        ))
    );
}

mod source_declaration;
mod source_invariance;
mod source_packet_records;
