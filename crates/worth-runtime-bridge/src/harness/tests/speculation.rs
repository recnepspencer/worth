use crate::facade::{
    BridgePreviewLifecycleStateKind, BridgePreviewSessionDeclaration,
    BridgePreviewSessionDeclarationIdentity, BridgeRequestKind, BridgeSignalBranchIdentity,
    BridgeSpeculativeBranchBinding, BridgeSpeculativeBranchBindingIdentity,
};
use crate::speculation::BridgePreviewSessionIdentity;

use super::support::{build_runtime, registration};
use crate::harness::fixtures::RecordingSignalBridgeSink;

fn preview_declaration() -> BridgePreviewSessionDeclaration {
    BridgePreviewSessionDeclaration::new(
        BridgePreviewSessionDeclarationIdentity::admit_bridge_owned("harness:preview-declaration"),
        BridgeRequestKind::Preview,
        BridgeSpeculativeBranchBinding::new(
            BridgeSpeculativeBranchBindingIdentity::admit_bridge_owned("harness:binding"),
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            BridgeSignalBranchIdentity::admit_bridge_owned("signal:harness"),
        ),
        crate::facade::BridgePreviewSessionBasis::new(
            crate::facade::BridgeTruthViewSelector::branch_snapshot(
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:harness"),
            ),
            crate::facade::BridgeSourceCapabilitySet::new(vec![
                crate::facade::BridgeSourceCapability::SnapshotRead,
                crate::facade::BridgeSourceCapability::BranchRead,
            ]),
            crate::facade::BridgePreviewRetainedArtifactSchema::PreviewLifecycleArtifactsV1,
        ),
    )
}

#[test]
fn bridge_harness_speculation_records_are_queryable_across_execution_promotion_and_replay() {
    let source =
        crate::relational_source::relational_test_support::empty_bridge_relational_source();
    let runtime = build_runtime(
        source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
    );
    let admitted = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned("harness:preview-session"),
            preview_declaration(),
        )
        .expect("preview declaration should admit");
    let (active, execution_record) = runtime.activate_preview_session(admitted, 3, 1, 2);
    let proof = active.promotion_admissibility_proof();
    let (_promoted, promotion_record) = runtime
        .promote_preview_session(active, &execution_record, &proof)
        .expect("promotion should succeed");
    let replay_bundle = runtime
        .replay_preview_bundle(&BridgePreviewSessionIdentity::admit_bridge_owned(
            "harness:preview-session",
        ))
        .expect("replay bundle should exist");
    let preview_session_identity =
        BridgePreviewSessionIdentity::admit_bridge_owned("harness:preview-session");

    let diagnostics = runtime.diagnostics();
    let handle = diagnostics.handle();

    assert_eq!(diagnostics.preview_execution_records().len(), 1);
    assert_eq!(diagnostics.preview_promotion_records().len(), 1);
    assert_eq!(
        diagnostics
            .preview_execution_record_for_session_identity(&preview_session_identity)
            .expect("execution record should be queryable by session identity")
            .record_identity(),
        execution_record.record_identity()
    );
    assert_eq!(
        handle
            .preview_promotion_record_for_session_identity(&preview_session_identity)
            .expect("promotion record should be queryable through diagnostics handle")
            .record_identity(),
        promotion_record.record_identity()
    );
    assert_eq!(
        diagnostics
            .explain_preview_replay_bundle(&replay_bundle)
            .lifecycle_outcome(),
        BridgePreviewLifecycleStateKind::Promoted
    );
}

#[test]
fn bridge_harness_speculation_discard_replay_remains_queryable() {
    let source =
        crate::relational_source::relational_test_support::empty_bridge_relational_source();
    let runtime = build_runtime(
        source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
    );
    let admitted = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned("harness:preview-discard"),
            preview_declaration(),
        )
        .expect("preview declaration should admit");
    let (active, execution_record) = runtime.activate_preview_session(admitted, 2, 1, 1);
    let (_discarded, discard_record) = runtime
        .discard_preview_session(active, &execution_record)
        .expect("discard should succeed");
    let replay_bundle = runtime
        .replay_preview_bundle(&BridgePreviewSessionIdentity::admit_bridge_owned(
            "harness:preview-discard",
        ))
        .expect("discard replay bundle should exist");

    let diagnostics = runtime.diagnostics();
    let discard_explanation = diagnostics
        .explain_last_preview_discard_record()
        .expect("discard explanation should exist");

    assert_eq!(diagnostics.preview_discard_records().len(), 1);
    assert_eq!(
        discard_explanation.preview_discard_record_identity(),
        discard_record.record_identity().as_str()
    );
    assert_eq!(
        diagnostics
            .explain_preview_replay_bundle(&replay_bundle)
            .lifecycle_outcome(),
        BridgePreviewLifecycleStateKind::Discarded
    );
}
