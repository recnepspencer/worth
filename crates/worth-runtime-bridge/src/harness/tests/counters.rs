use crate::facade::BridgeRouteRequest;

use super::support::{
    build_runtime_with_aspects, committed_patch, field_aspect_registration, field_slice_snapshot,
    registration,
};
use crate::facade::{
    BridgeDeliveryIntent, BridgePreviewResidueClass, BridgePreviewSessionDeclaration,
    BridgePreviewSessionDeclarationIdentity, BridgeReplayMode, BridgeRequestKind,
    BridgeSignalBranchIdentity, BridgeSourceCapability, BridgeSourceCapabilitySet,
    BridgeSpeculativeBranchBinding, BridgeSpeculativeBranchBindingIdentity,
    BridgeTruthViewSelector, HistoricalEvaluationDeclaration, SnapshotReadPacket,
    TruthBranchIdentity, TruthSnapshotIdentity,
};
use crate::harness::fixtures::{InMemoryRelationalBridgeSource, RecordingSignalBridgeSink};
use crate::speculation::BridgePreviewSessionIdentity;

struct CounterPreviewSessionBasisInput {
    truth_branch_identity: TruthBranchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
}

fn preview_session_basis(
    input: CounterPreviewSessionBasisInput,
) -> crate::facade::BridgePreviewSessionBasis {
    crate::facade::BridgePreviewSessionBasis::new(
        BridgeTruthViewSelector::branch_snapshot(
            input.truth_branch_identity,
            input.snapshot_identity,
        ),
        BridgeSourceCapabilitySet::new(vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::BranchRead,
        ]),
        crate::facade::BridgePreviewRetainedArtifactSchema::PreviewLifecycleArtifactsV1,
    )
}

#[test]
fn bridge_counters_expose_digest_input_bytes() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_snapshot(field_slice_snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let runtime = build_runtime_with_aspects(
        source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
        vec![field_aspect_registration()],
    );

    let result = runtime
        .deliver_invalidation(
            runtime
                .plan_committed_patch(
                    BridgeRouteRequest::for_commit(
                        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                    ),
                    execution,
                )
                .expect("route should plan before digest budget capture"),
            execution,
        )
        .expect("delivery should succeed before digest budget capture");

    assert!(result.counters().digest_computation_count() >= 8);
    assert!(result.counters().digest_input_bytes() > 0);
}

#[test]
fn historical_evaluation_counters_capture_selector_branch_and_materialization_width() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_snapshot(field_slice_snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let runtime = build_runtime_with_aspects(
        source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
        vec![field_aspect_registration()],
    );

    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::historical_commit(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        ),
        BridgeReplayMode::Enabled,
        runtime.policy().diagnostics_tier(),
        BridgeDeliveryIntent::PrepareSignalEvaluation,
    );
    let observation = runtime
        .materialize_truth_view_observation(
            runtime
                .plan_truth_view_packet(declaration, SnapshotReadPacket::new(vec![]), execution)
                .expect("historical declaration should plan"),
            execution,
        )
        .expect("historical declaration should materialize");
    let record = runtime.canonicalize_historical_evaluation_record(&observation);

    assert_eq!(record.counters().truth_view_selector_count(), 1);
    assert_eq!(record.counters().historical_truth_view_count(), 1);
    assert_eq!(record.counters().branch_truth_view_count(), 0);
    assert_eq!(record.counters().planned_truth_view_packet_count(), 1);
    assert_eq!(record.counters().resolved_truth_view_policy_count(), 1);
    assert_eq!(record.counters().materialized_truth_view_count(), 1);
    assert_eq!(record.counters().truth_view_decision_log_count(), 1);
    assert_eq!(record.counters().selector_width(), 1);
    assert_eq!(record.counters().branch_width(), 1);
    assert_eq!(record.counters().commit_envelope_materialization_count(), 1);
    assert_eq!(record.counters().direct_snapshot_materialization_count(), 0);
    assert_eq!(record.counters().branch_head_materialization_count(), 0);
}

#[test]
fn speculation_counters_capture_preview_discard_promotion_and_replay_widths() {
    let source = InMemoryRelationalBridgeSource::default();
    let runtime = build_runtime_with_aspects(
        source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
        vec![field_aspect_registration()],
    );
    let declaration = BridgePreviewSessionDeclaration::new(
        BridgePreviewSessionDeclarationIdentity::admit_bridge_owned("counter:preview-declaration"),
        BridgeRequestKind::Preview,
        BridgeSpeculativeBranchBinding::new(
            BridgeSpeculativeBranchBindingIdentity::admit_bridge_owned("counter:binding"),
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            BridgeSignalBranchIdentity::admit_bridge_owned("signal:counter"),
        ),
        preview_session_basis(CounterPreviewSessionBasisInput {
            truth_branch_identity: crate::truth_identity_fixtures::truth_branch_fixture("main"),
            snapshot_identity: crate::truth_identity_fixtures::truth_snapshot_fixture(
                "snapshot:counter",
            ),
        }),
    );

    let admitted = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned("counter:preview-session"),
            declaration,
        )
        .expect("preview declaration should admit");
    let (active, execution_record) = runtime.activate_preview_session(admitted, 4, 2, 2);

    assert_eq!(
        execution_record.counters().preview_session_count_touched(),
        1
    );
    assert_eq!(execution_record.counters().branch_binding_proof_width(), 2);
    assert_eq!(execution_record.counters().preview_artifact_count(), 4);
    assert_eq!(execution_record.counters().discard_artifact_count(), 2);
    assert_eq!(
        execution_record
            .counters()
            .retained_non_authoritative_artifact_count(),
        2
    );

    let (discarded, discard_record) = runtime
        .discard_preview_session(
            active,
            &execution_record,
            vec![
                BridgePreviewResidueClass::PreviewExecutionRetained,
                BridgePreviewResidueClass::ReplayRetainedNonAuthoritative,
                BridgePreviewResidueClass::TemporaryRoutingResidue,
            ],
        )
        .expect("discard should succeed");

    assert_eq!(discard_record.counters().preview_session_count_touched(), 1);
    assert_eq!(discard_record.counters().discard_artifact_count(), 1);
    assert_eq!(discard_record.counters().destroyed_artifact_count(), 1);
    assert_eq!(
        discard_record
            .counters()
            .retained_non_authoritative_artifact_count(),
        2
    );
    assert_eq!(discard_record.counters().replay_bundle_width(), 2);

    let replay_bundle = runtime
        .replay_preview_bundle(discarded.session_identity())
        .expect("replay bundle should exist");
    assert_eq!(replay_bundle.counters().preview_session_count_touched(), 1);
    assert_eq!(replay_bundle.counters().replay_bundle_width(), 2);

    let promotion_admitted = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned("counter:preview-promotion-session"),
            BridgePreviewSessionDeclaration::new(
                BridgePreviewSessionDeclarationIdentity::admit_bridge_owned(
                    "counter:preview-promotion-declaration",
                ),
                BridgeRequestKind::Preview,
                BridgeSpeculativeBranchBinding::new(
                    BridgeSpeculativeBranchBindingIdentity::admit_bridge_owned(
                        "counter:promotion-binding",
                    ),
                    crate::truth_identity_fixtures::truth_branch_fixture("main"),
                    BridgeSignalBranchIdentity::admit_bridge_owned("signal:counter"),
                ),
                preview_session_basis(CounterPreviewSessionBasisInput {
                    truth_branch_identity: crate::truth_identity_fixtures::truth_branch_fixture(
                        "main",
                    ),
                    snapshot_identity: crate::truth_identity_fixtures::truth_snapshot_fixture(
                        "snapshot:counter-promotion",
                    ),
                }),
            ),
        )
        .expect("promotion preview declaration should admit");
    let (promotion_active, promotion_execution_record) =
        runtime.activate_preview_session(promotion_admitted, 4, 2, 2);
    let proof = promotion_active.promotion_admissibility_proof();
    let (_promoted, promotion_record) = runtime
        .promote_preview_session(promotion_active, &promotion_execution_record, &proof)
        .expect("promotion should succeed");

    assert_eq!(
        promotion_record.counters().preview_session_count_touched(),
        1
    );
    assert_eq!(promotion_record.counters().branch_binding_proof_width(), 2);
    assert_eq!(promotion_record.counters().admissibility_proof_width(), 9);
    assert_eq!(promotion_record.counters().promotion_proof_checks(), 1);
    assert_eq!(promotion_record.counters().replay_bundle_width(), 2);
}
