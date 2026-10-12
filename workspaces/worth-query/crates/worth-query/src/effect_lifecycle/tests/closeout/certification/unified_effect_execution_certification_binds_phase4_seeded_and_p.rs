//! unified effect execution certification binds phase4 seeded and phase5 surfaces.
use super::*;

#[test]
fn unified_effect_execution_certification_binds_phase4_seeded_and_phase5_surfaces() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = certify_effect_execution_pipeline(request_execution);

    assert!(bundle
        .rows()
        .iter()
        .any(|row| { row.lane() == EffectExecutionCertificationLane::MutationReceiptSurface }));
    assert!(bundle
        .rows()
        .iter()
        .any(|row| { row.lane() == EffectExecutionCertificationLane::WritebackReceiptSurface }));
    assert!(bundle
        .rows()
        .iter()
        .any(|row| { row.lane() == EffectExecutionCertificationLane::BatchReceiptSurface }));
    assert!(bundle
        .rows()
        .iter()
        .any(|row| { row.lane() == EffectExecutionCertificationLane::AdvisorySurface }));
    assert!(bundle
        .rows()
        .iter()
        .any(|row| { row.lane() == EffectExecutionCertificationLane::DeferredSurface }));
    assert!(bundle
        .rows()
        .iter()
        .any(|row| { row.lane() == EffectExecutionCertificationLane::DeniedSurface }));
    assert!(bundle
        .rows()
        .iter()
        .any(|row| { row.lane() == EffectExecutionCertificationLane::MismatchDetectionSurface }));
    assert!(bundle
        .rows()
        .iter()
        .any(|row| { row.lane() == EffectExecutionCertificationLane::ProofShapeSurface }));
    assert!(bundle
        .rows()
        .iter()
        .any(|row| { row.lane() == EffectExecutionCertificationLane::PerformanceSurface }));
    assert!(bundle
        .rows()
        .iter()
        .any(|row| { row.lane() == EffectExecutionCertificationLane::SupportAndDxSurface }));
    assert!(bundle
        .rows()
        .iter()
        .any(|row| { row.lane() == EffectExecutionCertificationLane::CompileFailBoundary }));
    assert!(bundle
        .rows()
        .iter()
        .any(|row| { row.lane() == EffectExecutionCertificationLane::SeededReplayParity }));
    assert!(!bundle.seeded_bundle_digest().is_empty());
    assert!(!bundle.phase4_bundle_digest().is_empty());
    assert!(!bundle.certification_bundle_digest().is_empty());

    let deferred = bundle
        .rows()
        .iter()
        .find(|row| row.lane() == EffectExecutionCertificationLane::DeferredSurface)
        .expect("deferred lane should exist");
    let denied = bundle
        .rows()
        .iter()
        .find(|row| row.lane() == EffectExecutionCertificationLane::DeniedSurface)
        .expect("denied lane should exist");
    let mismatch = bundle
        .rows()
        .iter()
        .find(|row| row.lane() == EffectExecutionCertificationLane::MismatchDetectionSurface)
        .expect("mismatch lane should exist");
    let proof_shape = bundle
        .rows()
        .iter()
        .find(|row| row.lane() == EffectExecutionCertificationLane::ProofShapeSurface)
        .expect("proof-shape lane should exist");
    let performance = bundle
        .rows()
        .iter()
        .find(|row| row.lane() == EffectExecutionCertificationLane::PerformanceSurface)
        .expect("performance lane should exist");

    assert!(deferred.failure_digest().is_some());
    assert!(denied.failure_digest().is_some());
    assert!(mismatch.failure_digest().is_some());
    assert!(mismatch
        .evidence_detail()
        .contains("proof_shape|phase_progression|compile_fail_boundary|replay_parity"));
    assert!(proof_shape.evidence_detail().contains("proof_shape:"));
    assert!(proof_shape.evidence_detail().contains("phase_progression:"));
    assert_ne!(
        proof_shape.counter_snapshot_digest(),
        crate::effect_lifecycle::EffectLifecycleCounters::default().counter_for_reporting()
    );
    assert!(performance.evidence_detail().contains("normalization:"));
    assert!(performance.evidence_detail().contains("support:"));
    assert!(
        performance.counter_snapshot_digest()
            != crate::effect_lifecycle::EffectLifecycleCounters::default().counter_for_reporting()
    );
    assert_ne!(
        proof_shape.counter_snapshot_digest(),
        performance.counter_snapshot_digest()
    );

    let support_and_dx = bundle
        .rows()
        .iter()
        .find(|row| row.lane() == EffectExecutionCertificationLane::SupportAndDxSurface)
        .expect("support and dx lane should exist");
    for story in [
        "common_path_intent_authoring",
        "writeback_common_path",
        "inspectable_lowered_plan",
        "denial_or_rebind",
        "support_discovery",
        "batch_execution",
        "receipt_first_diagnostics",
    ] {
        assert!(
            support_and_dx.evidence_detail().contains(story),
            "missing transcript story {story}"
        );
    }
    assert!(support_and_dx
        .evidence_detail()
        .contains("basis()->effect(mutation)->using_basis(branch_head)->admit("));
    assert!(support_and_dx
        .evidence_detail()
        .contains("basis()->effect(writeback)->using_basis(tenant_scoped)->admit("));
    assert!(
        support_and_dx.counter_snapshot_digest()
            != crate::effect_lifecycle::EffectLifecycleCounters::default().counter_for_reporting()
    );
    assert_ne!(
        performance.counter_snapshot_digest(),
        support_and_dx.counter_snapshot_digest()
    );
}
