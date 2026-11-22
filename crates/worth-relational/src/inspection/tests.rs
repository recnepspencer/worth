use crate::facade::identity::KindId;
use crate::facade::inspection::{InspectionOrigin, InspectionScope};
use crate::facade::transactions::{
    EntityReference, MutationIntent, RecordRef, RelationMutationIntent,
    UpdateRelationEndpointsIntent, WorkerIntentBatch,
};
use crate::tests::support::*;

#[test]
fn structural_identity_origin_tracks_non_current_scope() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "scoped-identity");
    let version_id = runtime.current_version_id();
    let snapshot = runtime
        .visibility_authority()
        .snapshot()
        .expect("current snapshot fixture is admitted");

    let current = runtime
        .inspect_what_happened()
        .structural_identity(
            execution,
            InspectionScope::Current,
            RecordRef::Entity(entity),
        )
        .expect("read request remains live")
        .expect("current structural identity");
    let version = runtime
        .inspect_what_happened()
        .structural_identity(
            execution,
            InspectionScope::Version(version_id),
            RecordRef::Entity(entity),
        )
        .expect("read request remains live")
        .expect("version structural identity");
    let snapshot = runtime
        .inspect_what_happened()
        .structural_identity(
            execution,
            InspectionScope::Snapshot(snapshot),
            RecordRef::Entity(entity),
        )
        .expect("read request remains live")
        .expect("snapshot structural identity");

    assert_eq!(current.origin, InspectionOrigin::CurrentTruth);
    assert_eq!(version.origin, InspectionOrigin::VisibilitySnapshot);
    assert_eq!(snapshot.origin, InspectionOrigin::VisibilitySnapshot);
}

#[test]
fn historical_neighbors_follow_scoped_relation_endpoints_after_rewire() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let original_source = create_entity(execution, &runtime, "original-source");
    let shared_target = create_entity(execution, &runtime, "shared-target");
    let replacement_source = create_entity(execution, &runtime, "replacement-source");
    let relation = create_relation(execution, &runtime, original_source, shared_target, "edge");
    let historical_version = runtime.current_version_id();
    let historical_snapshot = runtime
        .visibility_authority()
        .snapshot()
        .expect("current snapshot fixture is admitted");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("rewire-edge").push(MutationIntent::Relation(
            RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                relation_id: relation,
                kind_id: KindId(2),
                source: EntityReference::Existing(replacement_source),
                target: EntityReference::Existing(shared_target),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(execution, &runtime)
        .expect("relation rewire should commit");

    let current_neighbors = runtime
        .inspect_what_happened()
        .neighbors(execution, InspectionScope::Current, original_source)
        .expect("read request remains live");
    let historical_neighbors = runtime
        .inspect_what_happened()
        .neighbors(
            execution,
            InspectionScope::Version(historical_version),
            original_source,
        )
        .expect("read request remains live");
    let snapshot_neighbors = runtime
        .inspect_what_happened()
        .neighbors(
            execution,
            InspectionScope::Snapshot(historical_snapshot),
            original_source,
        )
        .expect("read request remains live");

    assert!(current_neighbors.outgoing_relation_ids.is_empty());
    assert_eq!(historical_neighbors.outgoing_relation_ids, vec![relation]);
    assert_eq!(snapshot_neighbors.outgoing_relation_ids, vec![relation]);
}

#[test]
fn cumulative_fork_and_publication_byte_overflow_refuses_inspection_without_saturating() {
    use crate::facade::inspection::{
        RelationalBranchSharingInspectionDenial as Denial, RelationalMvccCostScope,
    };
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "fork-byte-overflow-seed");
    let main_id = runtime.main_branch_identity();
    let fork_branch = create_branch_from_main(&runtime, "fork-byte-overflow-target");
    let fork_id = runtime.branch_identity(&fork_branch).unwrap();
    let scope = RelationalMvccCostScope::capture(&runtime, vec![main_id.clone(), fork_id.clone()]);

    // Each branch's individual fork byte counter fits u64, but their sum overflows.
    runtime
        .history
        .branch_cell(main_id.branch_id())
        .unwrap()
        .publication_cell()
        .record_sharing_cost(|costs| {
            costs.fork_materialized_authoritative_bytes = u64::MAX - 32;
        });
    runtime
        .history
        .branch_cell(&fork_branch)
        .unwrap()
        .publication_cell()
        .record_sharing_cost(|costs| {
            costs.fork_materialized_authoritative_bytes = 64;
        });
    assert_eq!(
        runtime.observe_branch_sharing(execution, &[main_id.clone(), fork_id.clone()]),
        Err(Denial::AuthoritativeByteBoundOverflow)
    );
    assert_eq!(
        runtime.observe_mvcc_counters(&scope),
        Err(Denial::AuthoritativeByteBoundOverflow)
    );

    // Single-branch cumulative overflow via checked fork recording refuses instead of saturating.
    runtime
        .history
        .branch_cell(&fork_branch)
        .unwrap()
        .publication_cell()
        .record_sharing_cost_checked(|costs| {
            costs.fork_materialized_authoritative_bytes = costs
                .fork_materialized_authoritative_bytes
                .checked_add(u64::MAX)?;
            Some(())
        });
    assert_eq!(
        runtime.observe_branch_sharing(execution, &[fork_id]),
        Err(Denial::AuthoritativeByteBoundOverflow)
    );

    // Owner publication recorder (`record_root_publication`) refuses overflow
    // without advancing or saturating any of the recorded byte totals.
    let main_root = runtime
        .history
        .branch_cell(main_id.branch_id())
        .and_then(|cell| cell.root())
        .unwrap();
    let main_scope = RelationalMvccCostScope::capture(&runtime, vec![main_id.clone()]);
    let pub_branch = create_branch_from_main(&runtime, "pub-byte-overflow-target");
    let pub_id = runtime.branch_identity(&pub_branch).unwrap();
    let pub_scope = RelationalMvccCostScope::capture(&runtime, vec![pub_id.clone()]);
    runtime
        .history
        .branch_cell(&pub_branch)
        .unwrap()
        .publication_cell()
        .record_sharing_cost(|costs| {
            costs.copied_truth_bytes = 123;
            costs.publication_new_authoritative_bytes = u64::MAX - 4;
            costs.publication_new_root_authoritative_bytes = 456;
        });
    runtime
        .history
        .record_root_publication(&pub_branch, &main_root, Some(10));
    let snapshot_after = runtime.history.sharing_costs_for_branch(&pub_branch);
    assert_eq!(snapshot_after.copied_truth_bytes, 123);
    assert_eq!(snapshot_after.publication_new_authoritative_bytes, u64::MAX - 4);
    assert_eq!(snapshot_after.publication_new_root_authoritative_bytes, 456);
    assert_eq!(
        runtime.observe_branch_sharing(execution, &[pub_id]),
        Err(Denial::AuthoritativeByteBoundOverflow)
    );
    assert_eq!(
        runtime.observe_mvcc_counters(&pub_scope),
        Err(Denial::AuthoritativeByteBoundOverflow)
    );
    // Checked envelope-residency overflow (`None`) preserves all recorded byte
    // totals unchanged and refuses both sharing and MVCC counter inspections.
    let main_before = runtime.history.sharing_costs_for_branch(main_id.branch_id());
    runtime
        .history
        .record_root_publication(main_id.branch_id(), &main_root, None);
    let main_after = runtime.history.sharing_costs_for_branch(main_id.branch_id());
    assert_eq!(main_after.copied_truth_bytes, main_before.copied_truth_bytes);
    assert_eq!(
        main_after.publication_new_authoritative_bytes,
        main_before.publication_new_authoritative_bytes
    );
    assert_eq!(
        main_after.publication_new_root_authoritative_bytes,
        main_before.publication_new_root_authoritative_bytes
    );
    assert_eq!(
        runtime.observe_branch_sharing(execution, &[main_id]),
        Err(Denial::AuthoritativeByteBoundOverflow)
    );
    assert_eq!(
        runtime.observe_mvcc_counters(&main_scope),
        Err(Denial::AuthoritativeByteBoundOverflow)
    );
}
