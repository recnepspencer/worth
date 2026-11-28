use crate::facade::tests::runtime;
use crate::facade::tests::speculation::{preview_declaration, structural_basis};
use crate::facade::{BridgePreviewLifecycleStateKind, BridgeRuntimePolicy};
use crate::speculation::BridgePreviewSessionIdentity;

#[test]
fn runtime_replays_discarded_preview_bundle_from_retained_records() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let admitted = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:replay-discard"),
            preview_declaration(),
        )
        .expect("preview declaration should admit");
    let (active, execution_record) = runtime.activate_preview_session(admitted, 3, 2, 1);
    let (discarded, discard_record) = runtime
        .discard_preview_session(active, &execution_record)
        .expect("discard should succeed");

    let replay_bundle = runtime
        .replay_preview_bundle(discarded.session_identity())
        .expect("discarded preview should replay from retained records");

    assert_eq!(
        replay_bundle.lifecycle_outcome(),
        BridgePreviewLifecycleStateKind::Discarded
    );
    assert_eq!(
        replay_bundle
            .preview_discard_record()
            .expect("discard replay should contain discard record")
            .record_identity(),
        discard_record.record_identity()
    );
    assert_eq!(replay_bundle.counters().replay_bundle_width(), 2);
}

#[test]
fn runtime_rejects_post_discard_reentry_and_preserves_canonical_discard_bundle() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let admitted = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:discard-terminal"),
            preview_declaration(),
        )
        .expect("preview declaration should admit");
    let (active, execution_record) = runtime.activate_preview_session(admitted, 4, 2, 2);
    let (discarded, discard_record) = runtime
        .discard_preview_session(active, &execution_record)
        .expect("discard should succeed");

    let initial_replay = runtime
        .replay_preview_bundle(discarded.session_identity())
        .expect("discarded preview should replay");
    assert_eq!(
        initial_replay
            .preview_discard_record()
            .expect("discard replay should retain discard record")
            .record_identity(),
        discard_record.record_identity()
    );

    let hostile_reentry_error = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:discard-terminal"),
            preview_declaration().with_structural_basis(structural_basis(2)),
        )
        .expect_err("discarded preview session identity must reject hostile re-entry");
    assert_eq!(
        hostile_reentry_error.kind(),
        crate::error::BridgeSpeculationErrorKind::PreviewSessionIdentityConflict
    );

    let replay_after_reentry_attempt = runtime
        .replay_preview_bundle(discarded.session_identity())
        .expect("discard replay should remain canonical after rejected re-entry");

    assert_eq!(
        replay_after_reentry_attempt.lifecycle_outcome(),
        BridgePreviewLifecycleStateKind::Discarded
    );
    assert_eq!(
        replay_after_reentry_attempt
            .preview_discard_record()
            .expect("discard replay should still contain the original terminal record")
            .record_identity(),
        discard_record.record_identity()
    );
    assert!(
        replay_after_reentry_attempt
            .preview_promotion_record()
            .is_none(),
        "discarded sessions must not acquire promotion residue through hostile re-entry"
    );
    assert_eq!(runtime.diagnostics().preview_execution_records().len(), 1);
    assert_eq!(runtime.diagnostics().preview_discard_records().len(), 1);
    assert_eq!(runtime.diagnostics().preview_promotion_records().len(), 0);
}

#[test]
fn runtime_explains_preview_promotion_and_replay_from_retained_records() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let admitted = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:explain-promotion"),
            preview_declaration(),
        )
        .expect("preview declaration should admit");
    let (active, execution_record) = runtime.activate_preview_session(admitted, 5, 1, 2);
    let proof = active.promotion_admissibility_proof();
    let (_promoted, promotion_record) = runtime
        .promote_preview_session(active, &execution_record, &proof)
        .expect("promotion should succeed");

    let execution_explanation = runtime
        .diagnostics()
        .explain_last_preview_execution_record()
        .expect("execution explanation should exist");
    let promotion_explanation = runtime
        .diagnostics()
        .explain_last_preview_promotion_record()
        .expect("promotion explanation should exist");
    let replay_bundle = runtime
        .replay_preview_bundle(&BridgePreviewSessionIdentity::admit_bridge_owned(
            "preview-session:explain-promotion",
        ))
        .expect("promotion replay bundle should exist");
    let replay_explanation = runtime
        .diagnostics()
        .explain_preview_replay_bundle(&replay_bundle);

    assert_eq!(
        execution_explanation.preview_execution_record_identity(),
        execution_record.record_identity().as_str()
    );
    assert_eq!(
        promotion_explanation.preview_promotion_record_identity(),
        promotion_record.record_identity().as_str()
    );
    assert_eq!(
        replay_explanation.lifecycle_outcome(),
        BridgePreviewLifecycleStateKind::Promoted
    );
    assert!(replay_explanation.has_promotion_record());
}

#[test]
fn runtime_explains_preview_discard_from_retained_records() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let admitted = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:explain-discard"),
            preview_declaration(),
        )
        .expect("preview declaration should admit");
    let (active, execution_record) = runtime.activate_preview_session(admitted, 2, 1, 1);
    let (_discarded, discard_record) = runtime
        .discard_preview_session(active, &execution_record)
        .expect("discard should succeed");

    let discard_explanation = runtime
        .diagnostics()
        .explain_last_preview_discard_record()
        .expect("discard explanation should exist");

    assert_eq!(
        discard_explanation.preview_discard_record_identity(),
        discard_record.record_identity().as_str()
    );
    assert_eq!(discard_explanation.authoritative_residue_count(), 0);
    assert_eq!(discard_explanation.destroyed_artifact_count(), 0);
}
