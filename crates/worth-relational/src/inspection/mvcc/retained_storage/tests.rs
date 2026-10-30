//! Real owner custody and an independent allocation walk cross-examine the census.
use super::*;
use crate::inspection::mvcc::sharing::RelationalAuthoritativeAllocationKind as Kind;
use crate::tests::support::{create_entity_outcome, durable_runtime_with_test_schema};

#[test]
fn independent_readers_hold_history_until_last_release_then_queued_roots_reclaim() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let mut runtime = durable_runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "inventory-first");
    let independent = runtime.snapshots().snapshot().unwrap();
    let next = create_entity_outcome(execution, &runtime, "inventory-next");
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
        "classification happens before the inspector's own Arc clones"
    );
    assert_eq!(
        queued.authoritative_owner_footprint_bytes,
        held.authoritative_owner_footprint_bytes
    );
    assert_eq!(
        queued.catalog_owner_footprint_bytes,
        held.catalog_owner_footprint_bytes
    );
    let reclaimed = runtime.run_branch_root_reclamation_pass();
    assert!(reclaimed.roots_reclaimed() > 0);
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
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "inventory-shared-head");
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
    let branch = crate::history::data::BranchId("inventory-shared".into());
    let (_, source) = runtime.observe_fork_source(main.branch_id()).unwrap();
    runtime.fork_branch(execution, branch, source).unwrap();
    let after = runtime.observe_retained_storage().unwrap();
    assert_eq!(before.current_roots, 1);
    assert_eq!(after.current_roots, 1);
    assert_eq!(
        after.current_root_authoritative_bytes,
        before.current_root_authoritative_bytes
    );
}

#[test]
fn census_refuses_an_actual_branch_transition_in_flight() {
    let runtime = durable_runtime_with_test_schema();
    let transition = runtime.history.begin_branch_transition();
    assert_eq!(
        runtime.observe_retained_storage(),
        Err(RelationalRetainedStorageInspectionDenial::PublicationBusy)
    );
    drop(transition);
    assert!(runtime.observe_retained_storage().is_ok());
}

#[test]
fn equal_but_distinct_envelope_custody_does_not_count_as_catalog_linkage() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "inventory-linkage");
    let root = runtime
        .history
        .current_root_storage_inventory()
        .pop()
        .unwrap();
    let other = root
        .relink_canonical_envelope(
            Arc::new((**root.canonical_envelope().unwrap()).clone()),
            &runtime.services.symbols.interner_snapshot(),
        )
        .unwrap();
    let catalog = runtime
        .history
        .commit_artifacts()
        .into_iter()
        .map(|artifact| (artifact.commit_id(), artifact))
        .collect();
    let mut accounting = RelationalAuthoritativeAllocationAccounting::default();
    assert_eq!(
        observe_root(
            &runtime,
            &other,
            &catalog,
            &mut BTreeSet::new(),
            &mut accounting
        ),
        Err(RelationalBranchSharingInspectionDenial::RootUnavailable)
    );
    assert!(observe_root(
        &runtime,
        &root,
        &catalog,
        &mut BTreeSet::new(),
        &mut RelationalAuthoritativeAllocationAccounting::default()
    )
    .is_ok());
}
