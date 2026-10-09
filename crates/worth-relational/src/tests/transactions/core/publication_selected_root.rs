use crate::facade::history::BranchId;
use crate::facade::mvcc::{RelationalPublicationFailureKind, RelationalPublicationOutcome};
use crate::tests::support::{
    batch_create, runtime_with_test_schema, test_owner_begin_transaction_for_main,
};

#[test]
fn rootless_selected_publication_fails_before_movement_and_releases_preflight_residue() {
    let mut runtime = runtime_with_test_schema();
    runtime.set_retention_capacity_for_test(8, 1);
    let branch = BranchId("main".to_owned());
    let identity = runtime.main_branch_identity();
    let immutable_commit_count_before = runtime.history().immutable_commit_count();
    let commit_envelopes_before = runtime.history().commit_envelopes_snapshot();
    let pending_routes_before = runtime.history.pending_canonical_publication_route_count();
    let patch_stream_before = runtime
        .publication()
        .read_patch_stream(crate::publication::patch::data::PatchStreamRequest::default())
        .expect("selected-root evidence reads the canonical patch stream");
    let publication_bundle_before = runtime.publication().latest_bundle();
    let durable_count_before = runtime.durability().durable_log().len();
    let retention_before = runtime.retention().inspect_plan();
    let position_reservations_before = runtime.patch_position_reservation_counters();
    let retention_before_candidate = runtime
        .branch_retention_cost_counters(&identity)
        .expect("main branch retention counters remain observable");
    let registry_entries_before_candidate = runtime
        .branch_basis_cost_counters()
        .retained_basis_registry_entries;
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("selected-root-unavailable"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("candidate preparation succeeds before the injected owner fault");
    assert_eq!(candidate.reservation_count(), 1);
    let prepared_symbols = runtime.services.symbols.clone();
    let prepared_symbol_table = runtime.config().identity.symbol_table.clone();
    let retention_before_failure = runtime
        .branch_retention_cost_counters(&identity)
        .expect("main branch retention counters remain observable");
    runtime
        .history
        .branch_cell_mut(&branch)
        .expect("main branch remains registered")
        .clear_root_for_test();
    let rootless_reference = runtime
        .branch_reference_state(&branch)
        .expect("the injected rootless branch reference remains observable");

    let failure = match runtime.publication_port().compare_and_publish(candidate) {
        RelationalPublicationOutcome::Failed(failure) => failure,
        outcome => panic!("rootlessness must be a typed no-movement failure: {outcome:?}"),
    };

    assert_eq!(
        failure.kind(),
        &RelationalPublicationFailureKind::SelectedRootUnavailable
    );
    assert_eq!(
        runtime
            .branch_reference_state(&branch)
            .expect("rootless branch reference remains registered"),
        rootless_reference
    );
    assert_eq!(
        runtime.history().immutable_commit_count(),
        immutable_commit_count_before
    );
    assert_eq!(
        runtime.history().commit_envelopes_snapshot(),
        commit_envelopes_before
    );
    assert_eq!(
        runtime.history.pending_canonical_publication_route_count(),
        pending_routes_before
    );
    assert_eq!(
        runtime
            .publication()
            .read_patch_stream(crate::publication::patch::data::PatchStreamRequest::default())
            .expect("selected-root evidence rereads the canonical patch stream"),
        patch_stream_before
    );
    assert_eq!(
        runtime.publication().latest_bundle(),
        publication_bundle_before
    );
    assert_eq!(
        runtime.durability().durable_log().len(),
        durable_count_before
    );
    assert_eq!(runtime.retention().inspect_plan(), retention_before);
    assert_eq!(
        runtime.patch_position_reservation_counters(),
        position_reservations_before
    );
    assert_eq!(runtime.services.symbols, prepared_symbols);
    assert_eq!(
        runtime.config().identity.symbol_table,
        prepared_symbol_table
    );
    let retention_after_failure = runtime
        .branch_retention_cost_counters(&identity)
        .expect("rootless main branch keeps its retention owner");
    assert_eq!(
        retention_after_failure.candidate_releases,
        retention_before_failure.candidate_releases + 1
    );
    let observation_acquire_delta = retention_after_failure
        .observation_acquires
        .checked_sub(retention_before_candidate.observation_acquires)
        .expect("observation acquisition count is monotonic");
    let observation_release_delta = retention_after_failure
        .observation_releases
        .checked_sub(retention_before_candidate.observation_releases)
        .expect("observation release count is monotonic");
    assert!(
        observation_acquire_delta > 0,
        "publication preflight must acquire the next basis"
    );
    assert_eq!(observation_release_delta, observation_acquire_delta);
    assert_eq!(
        runtime
            .branch_basis_cost_counters()
            .retained_basis_registry_entries,
        registry_entries_before_candidate
    );

    let probe_root = crate::branch::RelationalBranchRoot::empty_with_schema(
        &runtime.config().schema.registry,
        crate::schema::data::runtime_descriptor_semantics_policy().current_write_version(),
    );
    let head_retention = std::sync::Arc::clone(
        runtime
            .history
            .branch_cell(&branch)
            .expect("rootless main branch keeps its head obligation")
            .head_retention(),
    );
    let retention_binding = head_retention
        .binding()
        .expect("rootless main branch keeps its retention owner");
    drop(
        retention_binding
            .reserve_head_retirement(&identity, &probe_root, &head_retention)
            .expect("failed preflight releases the single retirement slot"),
    );
}
