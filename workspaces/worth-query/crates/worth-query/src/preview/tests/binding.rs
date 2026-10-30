use crate::harness::fixtures::execution_preflights;
use crate::harness::fixtures::preview_bridge::{
    active_preview_artifacts, admitted_preview_session, declared_preview_session,
    discarded_preview_artifacts, promoted_preview_artifacts, promoted_preview_replay_bundle,
};
use crate::preview::{
    bind_preflight_to_preview_session, PreviewBindingFailureClass, PreviewEvaluationClass,
    PreviewSessionQueryContext,
};
use worth_runtime_bridge::facade::BridgePreviewLifecycleStateKind;

#[test]
fn active_preview_binding_succeeds_with_required_tuple_fields() {
    let preflight = execution_preflights::direct_runtime_preflight();
    let (_runtime, active, execution_record) = active_preview_artifacts("preview-active-success");
    let binding = bind_preflight_to_preview_session(
        preflight.clone(),
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::read_only(),
        ),
    )
    .expect("active preview should bind");

    assert_eq!(
        binding.basis().binding_tuple().preview_session_identity(),
        active.session_identity()
    );
    assert_eq!(
        binding.basis().binding_tuple().lifecycle_state_kind(),
        BridgePreviewLifecycleStateKind::Active
    );
    assert_eq!(
        binding.basis().binding_tuple().canonical_query_digest(),
        preflight.plan().query().canonical_query_digest()
    );
    assert_eq!(
        binding
            .basis()
            .binding_tuple()
            .canonical_result_shape_digest(),
        preflight
            .plan()
            .result_shape()
            .canonical_result_shape_digest()
    );
    assert_eq!(
        binding.basis().binding_tuple().evaluation_class(),
        &PreviewEvaluationClass::read_only()
    );
    assert_eq!(
        binding
            .report()
            .counters()
            .preview_lifecycle_rediscovery_count(),
        0
    );
    assert_eq!(
        binding
            .report()
            .counters()
            .preview_executor_rediscovery_count(),
        0
    );
}

#[test]
fn non_active_lifecycle_bindings_are_rejected() {
    let preflight = execution_preflights::direct_runtime_preflight();
    let (_declared_runtime, declared) = declared_preview_session("preview-declared-reject");
    let declared_error = bind_preflight_to_preview_session(
        preflight.clone(),
        PreviewSessionQueryContext::declared(&declared, PreviewEvaluationClass::read_only()),
    )
    .expect_err("declared preview should reject");
    assert_eq!(
        declared_error.failure_class(),
        &PreviewBindingFailureClass::StaleOrInactivePreviewLifecycle
    );

    let (_admitted_runtime, admitted) = admitted_preview_session("preview-admitted-reject");
    let admitted_error = bind_preflight_to_preview_session(
        preflight.clone(),
        PreviewSessionQueryContext::admitted(&admitted, PreviewEvaluationClass::read_only()),
    )
    .expect_err("admitted preview should reject");
    assert_eq!(
        admitted_error.failure_class(),
        &PreviewBindingFailureClass::StaleOrInactivePreviewLifecycle
    );

    let (_discarded_runtime, discarded, _) =
        discarded_preview_artifacts("preview-discarded-reject");
    let discarded_error = bind_preflight_to_preview_session(
        preflight.clone(),
        PreviewSessionQueryContext::discarded(&discarded, PreviewEvaluationClass::read_only()),
    )
    .expect_err("discarded preview should reject");
    assert_eq!(
        discarded_error.failure_class(),
        &PreviewBindingFailureClass::StaleOrInactivePreviewLifecycle
    );

    let (_promoted_runtime, promoted, _, _) = promoted_preview_artifacts("preview-promoted-reject");
    let promoted_error = bind_preflight_to_preview_session(
        preflight,
        PreviewSessionQueryContext::promoted(&promoted, PreviewEvaluationClass::read_only()),
    )
    .expect_err("promoted preview should reject");
    assert_eq!(
        promoted_error.failure_class(),
        &PreviewBindingFailureClass::StaleOrInactivePreviewLifecycle
    );
}

#[test]
fn preview_evaluation_classes_remain_distinct() {
    let read_only = PreviewEvaluationClass::read_only();
    let promotable = PreviewEvaluationClass::promotion_eligible();

    assert_ne!(read_only, promotable);
    assert_eq!(read_only.as_str(), "read_only");
    assert_eq!(promotable.as_str(), "promotion_eligible");
}

#[test]
fn missing_execution_record_identity_for_active_preview_rejects() {
    let preflight = execution_preflights::direct_runtime_preflight();
    let (_runtime, active, _execution_record) =
        active_preview_artifacts("preview-missing-execution-record");
    let error = bind_preflight_to_preview_session(
        preflight,
        PreviewSessionQueryContext::active_without_execution_record(
            &active,
            PreviewEvaluationClass::read_only(),
        ),
    )
    .expect_err("active preview without execution record should reject");

    assert_eq!(
        error.failure_class(),
        &PreviewBindingFailureClass::MissingExecutionRecordIdentity
    );
    assert_eq!(
        error.counters().preview_broad_fallback_denial_count(),
        0,
        "missing execution record should not masquerade as a broad-fallback denial"
    );
}

#[test]
fn store_backed_preflight_plus_preview_binding_rejects() {
    let preflight = execution_preflights::store_detail_preflight();
    let (_runtime, active, execution_record) = active_preview_artifacts("preview-store-preflight");
    let error = bind_preflight_to_preview_session(
        preflight,
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::read_only(),
        ),
    )
    .expect_err("store-backed preflight should reject");

    assert_eq!(
        error.failure_class(),
        &PreviewBindingFailureClass::StoreBackedRouteForbidden
    );
}

#[test]
fn binding_tuple_digest_is_stable_for_equivalent_admitted_inputs() {
    let left_preflight = execution_preflights::direct_runtime_preflight();
    let right_preflight = execution_preflights::replay_runtime_preflight();
    let (_runtime, active, execution_record) = active_preview_artifacts("preview-digest-stability");

    let left = bind_preflight_to_preview_session(
        left_preflight,
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::read_only(),
        ),
    )
    .expect("left preview binding should succeed");
    let right = bind_preflight_to_preview_session(
        right_preflight,
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::read_only(),
        ),
    )
    .expect("right preview binding should succeed");

    assert_eq!(
        left.basis().binding_tuple().digest(),
        right.basis().binding_tuple().digest()
    );
}

#[test]
fn evaluation_class_changes_binding_tuple_digest() {
    let preflight = execution_preflights::direct_runtime_preflight();
    let (_runtime, active, execution_record) =
        active_preview_artifacts("preview-evaluation-class-digest");

    let read_only = bind_preflight_to_preview_session(
        preflight.clone(),
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::read_only(),
        ),
    )
    .expect("read-only preview binding should succeed");
    let promotable = bind_preflight_to_preview_session(
        preflight,
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::promotion_eligible(),
        ),
    )
    .expect("promotion-eligible preview binding should succeed");

    assert_ne!(
        read_only.basis().binding_tuple().digest(),
        promotable.basis().binding_tuple().digest()
    );
}

#[test]
fn promotion_linkage_rejects_even_for_promotion_eligible_active_binding() {
    let preflight = execution_preflights::direct_runtime_preflight();
    let (_runtime, active, execution_record) =
        active_preview_artifacts("preview-promotion-linkage-denied");
    let (_promoted_runtime, _promoted, _promoted_execution, promotion_record) =
        promoted_preview_artifacts("preview-promotion-linkage-source");

    let error = bind_preflight_to_preview_session(
        preflight,
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::promotion_eligible(),
        )
        .with_promotion_record(&promotion_record),
    )
    .expect_err("promotion linkage should reject for phase 1-2 active binding");

    assert_eq!(
        error.failure_class(),
        &PreviewBindingFailureClass::PromotionLinkageMismatch
    );
    assert_eq!(error.counters().preview_bridge_promotion_linkage_count(), 1);
    assert_eq!(error.counters().preview_replay_bundle_lookup_count(), 0);
}

#[test]
fn replay_bundle_rejects_for_active_binding() {
    let preflight = execution_preflights::direct_runtime_preflight();
    let (_runtime, active, execution_record) =
        active_preview_artifacts("preview-replay-linkage-denied");
    let (_promoted_runtime, _promoted, _promoted_execution, _promotion_record, replay_bundle) =
        promoted_preview_replay_bundle("preview-replay-linkage-source");

    let error = bind_preflight_to_preview_session(
        preflight,
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::promotion_eligible(),
        )
        .with_replay_bundle(&replay_bundle),
    )
    .expect_err("replay linkage should reject for phase 1-2 active binding");

    assert_eq!(
        error.failure_class(),
        &PreviewBindingFailureClass::PromotionLinkageMismatch
    );
    assert_eq!(error.counters().preview_replay_bundle_lookup_count(), 1);
    assert_eq!(error.counters().preview_bridge_promotion_linkage_count(), 0);
}

#[test]
fn unsupported_preview_query_families_are_rejected() {
    let (_runtime, active, execution_record) =
        active_preview_artifacts("preview-unsupported-families");

    for preflight in [
        execution_preflights::cdc_collection_preflight(),
        execution_preflights::aggregate_rollup_collection_preflight(),
    ] {
        let error = bind_preflight_to_preview_session(
            preflight,
            PreviewSessionQueryContext::active(
                &active,
                &execution_record,
                PreviewEvaluationClass::read_only(),
            ),
        )
        .expect_err("unsupported preview family should reject");

        assert_eq!(
            error.failure_class(),
            &PreviewBindingFailureClass::UnsupportedPreviewQueryFamily
        );
    }
}
