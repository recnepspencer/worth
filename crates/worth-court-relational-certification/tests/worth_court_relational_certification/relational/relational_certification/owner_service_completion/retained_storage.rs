//! Census union, reader custody and canonical admission through public owners.
use crate::relational_certification::ordinary_context::owner_service_completion::world::{
    commit_through_services, empty_runtime, fork_through_services,
};
use std::collections::BTreeMap;
use worth_relational::facade::{
    inspection::{RelationalAuthoritativeAllocationKind as Kind, RelationalMvccCostScope},
    mvcc::RelationalTransactionIntent,
    runtime::RelationalRuntime,
    transactions::WorkerIntentBatch,
};

#[test]
fn independent_readers_hold_history_until_last_release_then_queued_roots_reclaim() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let mut runtime = empty_runtime();
    let services = runtime.owner_component_services();
    let first = commit_through_services(execution, &runtime, &services, "inventory-first");
    let independent = {
        let basis = runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap();
        runtime
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap()
    };
    let next = commit_through_services(execution, &runtime, &services, "inventory-next");
    runtime
        .snapshots()
        .release_snapshot(&next.snapshot)
        .unwrap();
    let held = runtime.observe_retained_storage().unwrap();
    assert!(held.held_roots > 0 && held.held_history_only_authoritative_bytes > 0);
    runtime
        .snapshots()
        .release_snapshot(&first.snapshot)
        .unwrap();
    let one_reader = runtime.observe_retained_storage().unwrap();
    assert_eq!(held.held_roots, one_reader.held_roots);
    assert_eq!(
        held.held_history_only_authoritative_bytes,
        one_reader.held_history_only_authoritative_bytes
    );
    runtime.snapshots().release_snapshot(&independent).unwrap();
    let queued = runtime.observe_retained_storage().unwrap();
    assert!(queued.held_roots < one_reader.held_roots);
    assert!(queued.reclaimable_roots > one_reader.reclaimable_roots);
    assert!(
        queued.reclaimable_only_authoritative_bytes > 0,
        "classification precedes inspector-created Arc clones"
    );
    assert_eq!(
        queued.authoritative_owner_footprint_bytes,
        held.authoritative_owner_footprint_bytes
    );
    assert_eq!(
        queued.catalog_owner_footprint_bytes,
        held.catalog_owner_footprint_bytes
    );
    drop(services);
    assert!(runtime.run_branch_root_reclamation_pass().roots_reclaimed() > 0);
    let after = runtime.observe_retained_storage().unwrap();
    assert!(after.authoritative_owner_footprint_bytes < queued.authoritative_owner_footprint_bytes);
    assert_eq!(
        after.current_root_authoritative_bytes,
        queued.current_root_authoritative_bytes
    );
    assert_eq!(
        after.catalog_owner_footprint_bytes, queued.catalog_owner_footprint_bytes,
        "reader release does not reclaim the append-only catalog"
    );
    assert_eq!(
        after.authoritative_owner_footprint_bytes,
        after.current_root_authoritative_bytes
            + after.held_history_only_authoritative_bytes
            + after.reclaimable_only_authoritative_bytes
            + after.catalog_owner_footprint_bytes
    );
}

#[test]
fn shared_heads_deduplicate_current_root_bytes_against_the_independent_owner_walk() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = empty_runtime();
    let services = runtime.owner_component_services();
    let committed =
        commit_through_services(execution, &runtime, &services, "inventory-shared-head");
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    let before = runtime.observe_retained_storage().unwrap();
    let main = runtime.main_branch_identity();
    let owner = runtime
        .inspect_owner_allocation_ledger(std::slice::from_ref(&main))
        .unwrap();
    let independent = owner
        .authoritative_allocations()
        .iter()
        .filter(|row| {
            !matches!(
                row.locator().kind(),
                Kind::CanonicalCommitArtifact
                    | Kind::CanonicalCommitPayload
                    | Kind::CanonicalCommitEnvelope
                    | Kind::CanonicalCommitEnvelopeNested
            )
        })
        .map(|row| (row.locator(), row.authoritative_bytes()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        before.current_root_authoritative_bytes,
        independent.values().copied().sum::<u64>()
    );
    fork_through_services(execution, &services, "inventory-shared");
    let after = runtime.observe_retained_storage().unwrap();
    assert_eq!(before.current_roots, 1);
    assert_eq!(after.current_roots, 1);
    assert_eq!(
        after.current_root_authoritative_bytes,
        before.current_root_authoritative_bytes
    );
}

#[test]
fn settled_history_and_dropped_reservations_leave_no_admission_candidates() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = empty_runtime();
    let services = runtime.owner_component_services();
    for index in 0..100 {
        let committed = commit_through_services(
            execution,
            &runtime,
            &services,
            &format!("settled-route-{index}"),
        );
        runtime
            .snapshots()
            .release_snapshot(&committed.snapshot)
            .unwrap();
    }
    assert_eq!(census_admission_visits(&runtime), 0);
    let basis = services
        .basis_port()
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("reserved-unperformed-route"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = services
        .preparation_port()
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    assert_eq!(census_admission_visits(&runtime), 1);
    drop(candidate);
    assert_eq!(census_admission_visits(&runtime), 0);
}

fn census_admission_visits(runtime: &RelationalRuntime) -> u64 {
    let scope = RelationalMvccCostScope::capture(runtime, vec![]);
    runtime.observe_retained_storage().unwrap();
    runtime
        .observe_mvcc_counters(&scope)
        .unwrap()
        .canonical_settlement_route_visits()
}
