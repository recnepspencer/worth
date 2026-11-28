use super::super::support::registration;
use crate::facade::{
    BridgePreviewSessionDeclaration, BridgePreviewSessionDeclarationIdentity, BridgeRequestKind,
    BridgeSignalBranchIdentity, BridgeSourceCapability, BridgeSourceCapabilitySet,
    BridgeSpeculativeBranchBinding, BridgeSpeculativeBranchBindingIdentity,
    BridgeTruthViewSelector,
};
use crate::speculation::BridgePreviewSessionIdentity;

#[test]
fn bridge_speculation_promotion_truth_is_invariant_across_diagnostics_tiers() {
    let source =
        crate::relational_source::relational_test_support::empty_bridge_relational_source();
    let baseline_runtime = crate::facade::RuntimeBridge::builder()
        .with_relational_source(source.clone())
        .with_truth_branch_head_source(source.clone())
        .with_signal_sink(crate::harness::fixtures::RecordingSignalBridgeSink::default())
        .with_policy(crate::facade::BridgeRuntimePolicy::development())
        .register_mapping(registration())
        .build()
        .expect("baseline speculation runtime should build");
    let forensic_runtime = crate::facade::RuntimeBridge::builder()
        .with_relational_source(source.clone())
        .with_truth_branch_head_source(source)
        .with_signal_sink(crate::harness::fixtures::RecordingSignalBridgeSink::default())
        .with_policy(crate::facade::BridgeRuntimePolicy::forensic())
        .register_mapping(registration())
        .build()
        .expect("forensic speculation runtime should build");

    let baseline_decl = BridgePreviewSessionDeclaration::new(
        BridgePreviewSessionDeclarationIdentity::admit_bridge_owned("cert:preview-declaration"),
        BridgeRequestKind::Preview,
        BridgeSpeculativeBranchBinding::new(
            BridgeSpeculativeBranchBindingIdentity::admit_bridge_owned("cert:binding"),
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            BridgeSignalBranchIdentity::admit_bridge_owned("signal:cert"),
        ),
        crate::facade::BridgePreviewSessionBasis::new(
            BridgeTruthViewSelector::branch_snapshot(
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:cert"),
            ),
            BridgeSourceCapabilitySet::new(vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::BranchRead,
            ]),
            crate::facade::BridgePreviewRetainedArtifactSchema::PreviewLifecycleArtifactsV1,
        ),
    );
    let forensic_decl = baseline_decl.clone();

    let baseline_admitted = baseline_runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned("cert:preview-session"),
            baseline_decl,
        )
        .expect("baseline declaration should admit");
    let forensic_admitted = forensic_runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned("cert:preview-session"),
            forensic_decl,
        )
        .expect("forensic declaration should admit");

    let (baseline_active, baseline_execution) =
        baseline_runtime.activate_preview_session(baseline_admitted, 3, 1, 2);
    let (forensic_active, forensic_execution) =
        forensic_runtime.activate_preview_session(forensic_admitted, 3, 1, 2);

    let baseline_proof = baseline_active.promotion_admissibility_proof();
    let forensic_proof = forensic_active.promotion_admissibility_proof();

    let (_baseline_promoted, baseline_promotion) = baseline_runtime
        .promote_preview_session(baseline_active, &baseline_execution, &baseline_proof)
        .expect("baseline promotion should succeed");
    let (_forensic_promoted, forensic_promotion) = forensic_runtime
        .promote_preview_session(forensic_active, &forensic_execution, &forensic_proof)
        .expect("forensic promotion should succeed");

    let baseline_replay = baseline_runtime
        .replay_preview_bundle(&BridgePreviewSessionIdentity::admit_bridge_owned(
            "cert:preview-session",
        ))
        .expect("baseline replay should succeed");
    let forensic_replay = forensic_runtime
        .replay_preview_bundle(&BridgePreviewSessionIdentity::admit_bridge_owned(
            "cert:preview-session",
        ))
        .expect("forensic replay should succeed");

    assert_eq!(baseline_execution.digest(), forensic_execution.digest());
    assert_eq!(baseline_promotion.digest(), forensic_promotion.digest());
    assert_eq!(baseline_replay.digest(), forensic_replay.digest());
    assert_eq!(
        baseline_runtime
            .diagnostics()
            .explain_preview_replay_bundle(&baseline_replay)
            .lifecycle_outcome(),
        forensic_runtime
            .diagnostics()
            .explain_preview_replay_bundle(&forensic_replay)
            .lifecycle_outcome()
    );
}
