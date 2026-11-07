use super::super::{
    admit_and_scope_legacy_query_basis_context_for_test, attach_query_basis_metadata,
    bind_legacy_query_basis_context, execute_query_basis_context, QueryBasisContextRequest,
    QueryContextAdmissionFailureClass, QueryContextBindingSource,
};
use crate::facade::foundation::{
    admit_historical_evaluation_path, materialization_metadata_from_resolved,
    resolve_historical_materialization_path, HistoricalCapabilityDescriptor,
    HistoricalEvaluationRequest, HistoricalMaterializationDescriptor,
    HistoricalPathReuseDescriptor,
};
use crate::harness::fixtures::execution_preflights;

#[test]
fn invalid_runtime_current_vs_branch_pairing_rejects_typed_and_early() {
    let preflight = execution_preflights::direct_runtime_preflight();
    let error = bind_legacy_query_basis_context(
        QueryBasisContextRequest::branch_head("branch:snapshot-2"),
        QueryContextBindingSource::RuntimeCurrent(&preflight),
    )
    .expect_err("runtime current source cannot bind branch-head family");

    assert_eq!(
        error.failure_class(),
        &QueryContextAdmissionFailureClass::InvalidBasisPairing
    );
    assert_eq!(error.counters().query_basis_binding_count(), 0);
    assert_eq!(error.counters().basis_binding_width(), 0);
    assert_eq!(error.counters().denial_width(), 1);
}

#[test]
fn store_backed_historical_debt_is_denied_typed_and_early() {
    let capability = HistoricalCapabilityDescriptor::new_for_test(
        "history:store",
        None,
        false,
        false,
        false,
        true,
        HistoricalPathReuseDescriptor::no_reuse(),
    )
    .expect("fixture digest must fit");
    let error = bind_legacy_query_basis_context(
        QueryBasisContextRequest::historical_commit("history:store"),
        QueryContextBindingSource::HistoricalCapability(&capability),
    )
    .expect_err("store-backed history should remain deferred debt");

    assert_eq!(
        error.failure_class(),
        &QueryContextAdmissionFailureClass::StoreBackedHistoricalDeferred
    );
    assert_eq!(error.counters().unsupported_basis_denial_count(), 1);
    assert_eq!(error.counters().historical_lookup_width(), 1);
    assert_eq!(error.counters().denial_width(), 1);
}

#[test]
fn store_backed_retained_historical_binding_preserves_query_owned_parity() {
    let runtime_preflight = execution_preflights::direct_runtime_preflight();
    let store_preflight = execution_preflights::store_detail_preflight();
    let request = HistoricalEvaluationRequest::retained_snapshot_for_test(
        "history:snapshot-1",
        1,
        1,
        HistoricalPathReuseDescriptor::retained_reuse(),
    )
    .expect("fixture digest must fit");
    let capability = HistoricalCapabilityDescriptor::retained_snapshot_for_test(
        "history:snapshot-1",
        HistoricalPathReuseDescriptor::retained_reuse(),
    )
    .expect("fixture digest must fit");
    let admission = admit_historical_evaluation_path(request, capability)
        .expect("retained history should admit");
    let resolved = resolve_historical_materialization_path(
        admission.clone(),
        HistoricalMaterializationDescriptor::retained_snapshot_for_test("history:snapshot-1")
            .expect("fixture digest must fit"),
    )
    .expect("retained history should resolve");
    let metadata = materialization_metadata_from_resolved(resolved);

    let runtime = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::historical_snapshot("history:snapshot-1"),
            QueryContextBindingSource::Historical {
                query_preflight: &runtime_preflight,
                admission: &admission,
                metadata: &metadata,
            },
        )
        .expect("runtime historical context should bind"),
    )
    .expect("runtime historical context should admit");
    let store = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::historical_snapshot("history:snapshot-1"),
            QueryContextBindingSource::Historical {
                query_preflight: &store_preflight,
                admission: &admission,
                metadata: &metadata,
            },
        )
        .expect("store historical context should bind"),
    )
    .expect("store historical context should admit");

    let runtime_bundle = attach_query_basis_metadata(
        &runtime,
        &execute_query_basis_context(&runtime).expect("runtime execution should succeed"),
    )
    .expect("runtime metadata should shape");
    let store_bundle = attach_query_basis_metadata(
        &store,
        &execute_query_basis_context(&store).expect("store execution should succeed"),
    )
    .expect("store metadata should shape");

    assert_eq!(
        runtime_bundle.result_digest(),
        store_bundle.result_digest(),
        "store-backed retained history must preserve canonical result parity"
    );
    assert_eq!(
        runtime_bundle.materialization_path_identity(),
        store_bundle.materialization_path_identity()
    );
    assert_eq!(
        runtime_bundle.historical_admission_class(),
        store_bundle.historical_admission_class()
    );
    assert_eq!(
        runtime.basis_authority_family(),
        &crate::basis::BasisAuthorityFamily::Runtime
    );
    assert_eq!(
        store.basis_authority_family(),
        &crate::basis::BasisAuthorityFamily::Store
    );
}

#[test]
fn store_backed_replay_historical_binding_stays_explicit_deferred_debt() {
    let store_preflight = execution_preflights::store_detail_preflight();
    let request = HistoricalEvaluationRequest::delta_replay_for_test(
        "history:snapshot-1",
        1,
        1,
        HistoricalPathReuseDescriptor::no_reuse(),
    )
    .expect("fixture digest must fit");
    let capability = HistoricalCapabilityDescriptor::delta_replay_for_test(
        "history:snapshot-1",
        HistoricalPathReuseDescriptor::no_reuse(),
    )
    .expect("fixture digest must fit");
    let admission =
        admit_historical_evaluation_path(request, capability).expect("replay history should admit");
    let resolved = resolve_historical_materialization_path(
        admission.clone(),
        HistoricalMaterializationDescriptor::delta_replay_for_test("history:snapshot-1")
            .expect("fixture digest must fit"),
    )
    .expect("replay history should resolve");
    let metadata = materialization_metadata_from_resolved(resolved);

    let error = bind_legacy_query_basis_context(
        QueryBasisContextRequest::historical_snapshot("history:snapshot-1"),
        QueryContextBindingSource::Historical {
            query_preflight: &store_preflight,
            admission: &admission,
            metadata: &metadata,
        },
    )
    .expect_err("store-backed replay must remain deferred until a later milestone");

    assert_eq!(
        error.failure_class(),
        &QueryContextAdmissionFailureClass::StoreBackedHistoricalDeferred
    );
    assert_eq!(error.counters().unsupported_basis_denial_count(), 1);
    assert_eq!(error.counters().historical_lookup_width(), 1);
    assert_eq!(error.counters().denial_width(), 1);
}

#[test]
fn store_backed_reconstruction_historical_binding_stays_explicit_deferred_debt() {
    let store_preflight = execution_preflights::store_detail_preflight();
    let request = HistoricalEvaluationRequest::full_reconstruction_for_test(
        "history:snapshot-1",
        1,
        1,
        HistoricalPathReuseDescriptor::no_reuse(),
    )
    .expect("fixture digest must fit");
    let capability = HistoricalCapabilityDescriptor::full_reconstruction_for_test(
        "history:snapshot-1",
        HistoricalPathReuseDescriptor::no_reuse(),
    )
    .expect("fixture digest must fit");
    let admission = admit_historical_evaluation_path(request, capability)
        .expect("reconstruction history should admit");
    let resolved = resolve_historical_materialization_path(
        admission.clone(),
        HistoricalMaterializationDescriptor::full_reconstruction_for_test("history:snapshot-1")
            .expect("fixture digest must fit"),
    )
    .expect("reconstruction history should resolve");
    let metadata = materialization_metadata_from_resolved(resolved);

    let error = bind_legacy_query_basis_context(
        QueryBasisContextRequest::historical_snapshot("history:snapshot-1"),
        QueryContextBindingSource::Historical {
            query_preflight: &store_preflight,
            admission: &admission,
            metadata: &metadata,
        },
    )
    .expect_err("store-backed reconstruction must remain deferred until a later milestone");

    assert_eq!(
        error.failure_class(),
        &QueryContextAdmissionFailureClass::StoreBackedHistoricalDeferred
    );
    assert_eq!(error.counters().unsupported_basis_denial_count(), 1);
    assert_eq!(error.counters().historical_lookup_width(), 1);
    assert_eq!(error.counters().denial_width(), 1);
}
