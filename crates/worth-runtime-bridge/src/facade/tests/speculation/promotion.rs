use crate::facade::tests::runtime;
use crate::facade::tests::speculation::preview_declaration;
use crate::facade::{
    BridgePreviewLifecycleStateKind, BridgePreviewResidueClass, BridgeRuntimePolicy,
};
use crate::speculation::BridgePreviewSessionIdentity;

#[test]
fn runtime_promotes_preview_session_and_replays_promoted_bundle() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let admitted = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:promotion"),
            preview_declaration(),
        )
        .expect("preview declaration should admit");
    let (active, execution_record) = runtime.activate_preview_session(admitted, 3, 1, 2);
    let proof = active.promotion_admissibility_proof();

    let (promoted, promotion_record) = runtime
        .promote_preview_session(active, &execution_record, &proof)
        .expect("promotion should succeed");
    let replay_bundle = runtime
        .replay_preview_bundle(promoted.session_identity())
        .expect("retained promotion records should replay");

    assert_eq!(
        promoted.lifecycle_state_kind(),
        BridgePreviewLifecycleStateKind::Promoted
    );
    assert_eq!(runtime.diagnostics().preview_promotion_records().len(), 1);
    assert_eq!(
        replay_bundle.lifecycle_outcome(),
        BridgePreviewLifecycleStateKind::Promoted
    );
    assert_eq!(
        replay_bundle
            .preview_promotion_record()
            .expect("promotion replay should contain terminal record")
            .record_identity(),
        promotion_record.record_identity()
    );
}

#[test]
fn runtime_rejects_stale_duplicate_and_post_discard_promotion() {
    let runtime = runtime(BridgeRuntimePolicy::default());

    let admitted_a = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:promotion-a"),
            preview_declaration(),
        )
        .expect("first preview declaration should admit");
    let (active_a, execution_record_a) = runtime.activate_preview_session(admitted_a, 2, 1, 1);
    let stale_proof = active_a.promotion_admissibility_proof();

    let admitted_b = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:promotion-b"),
            preview_declaration(),
        )
        .expect("second preview declaration should admit");
    let (active_b_stale, execution_record_b_stale) =
        runtime.activate_preview_session(admitted_b, 2, 1, 1);

    let original_digest = active_b_stale.digest().to_owned();
    let (active_b, stale_error) = runtime
        .promote_preview_session(active_b_stale, &execution_record_b_stale, &stale_proof)
        .expect_err("stale proof must fail promotion");
    assert_eq!(
        stale_error.kind(),
        crate::error::BridgeSpeculationErrorKind::PromotionAdmissibilityMismatch
    );
    assert_eq!(active_b.digest(), original_digest);
    assert!(runtime.diagnostics().preview_promotion_records().is_empty());
    let actual_proof = active_b.promotion_admissibility_proof();
    let (promoted, _) = runtime
        .promote_preview_session(active_b, &execution_record_b_stale, &actual_proof)
        .unwrap();
    assert_eq!(
        promoted.session_identity().as_str(),
        "preview-session:promotion-b"
    );
    assert_eq!(runtime.diagnostics().preview_execution_records().len(), 2);
    assert_eq!(runtime.diagnostics().preview_promotion_records().len(), 1);

    let duplicate_admission_error = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:promotion-b"),
            preview_declaration(),
        )
        .expect_err("duplicate preview session identity must fail at admission");
    assert_eq!(
        duplicate_admission_error.kind(),
        crate::error::BridgeSpeculationErrorKind::PreviewSessionIdentityConflict
    );

    let (_discarded, _discard_record) = runtime
        .discard_preview_session(
            active_a,
            &execution_record_a,
            vec![
                BridgePreviewResidueClass::PreviewExecutionRetained,
                BridgePreviewResidueClass::TemporaryRoutingResidue,
            ],
        )
        .expect("discard should succeed");
    let reactivated_admission_error = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:promotion-a"),
            preview_declaration(),
        )
        .expect_err("discarded preview session identity must remain reserved");
    assert_eq!(
        reactivated_admission_error.kind(),
        crate::error::BridgeSpeculationErrorKind::PreviewSessionIdentityConflict
    );
}
