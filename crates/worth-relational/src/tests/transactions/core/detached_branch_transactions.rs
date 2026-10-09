use super::detached_branch_footprint_assertion::assert_detached_branch_footprint;
use crate::facade::history::BranchId;
use crate::facade::transactions::{
    CreateIntent, CreatedEntityRef, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};
use crate::tests::support::*;
use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;

#[test]
fn branch_transactions_are_detached_and_overlays_do_not_cross() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "fork-source");
    fork_from_main(&mut runtime, "storm");
    fork_from_main(&mut runtime, "maintenance");

    let (storm_basis, mut storm) = begin_on(&runtime, "storm");
    let (maintenance_basis, mut maintenance) = begin_on(&runtime, "maintenance");
    let storm_created = created_entity("storm-write");
    let maintenance_created = created_entity("maintenance-write");
    let storm_batch = batch_create("storm-write");
    let storm_expected = create_intent(&storm_batch);
    storm
        .push_batch(storm_batch, AllocationPolicy::SystemAllocation)
        .expect("test staging stays within configured resource budgets");
    let maintenance_batch = batch_create("maintenance-write");
    let maintenance_expected = create_intent(&maintenance_batch);
    maintenance
        .push_batch(maintenance_batch, AllocationPolicy::SystemAllocation)
        .expect("test staging stays within configured resource budgets");

    assert_eq!(
        storm
            .read_created_entity(&storm_created)
            .expect("read footprint fits")
            .expect("storm observes its exact staged create")
            .cloned()
            .collect::<Vec<_>>(),
        vec![storm_expected]
    );
    assert!(storm
        .read_created_entity(&maintenance_created)
        .unwrap()
        .is_none());
    assert_eq!(
        maintenance
            .read_created_entity(&maintenance_created)
            .expect("read footprint fits")
            .expect("maintenance observes its exact staged create")
            .cloned()
            .collect::<Vec<_>>(),
        vec![maintenance_expected]
    );
    assert!(maintenance
        .read_created_entity(&storm_created)
        .unwrap()
        .is_none());
    assert_detached_branch_footprint(
        &storm_basis,
        storm.footprint(),
        &storm_created,
        &maintenance_created,
    );
    assert_detached_branch_footprint(
        &maintenance_basis,
        maintenance.footprint(),
        &maintenance_created,
        &storm_created,
    );

    create_entity(&runtime, "unrelated-main-work");
    let storm_commit = storm
        .commit(&runtime, AllocationPolicy::SystemAllocation)
        .expect("storm commits independently");
    let maintenance_commit = maintenance
        .commit(&runtime, AllocationPolicy::SystemAllocation)
        .expect("maintenance commits independently");
    assert_eq!(storm_commit.commit.branch_id, BranchId("storm".into()));
    assert_eq!(
        maintenance_commit.commit.branch_id,
        BranchId("maintenance".into())
    );
}

#[test]
fn validated_proposal_complexity_excludes_intervening_sibling_commit_work() {
    let mut runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "relation-source");
    let target = create_entity(&runtime, "relation-target");
    fork_from_main(&mut runtime, "candidate");
    fork_from_main(&mut runtime, "sibling");

    let (_, mut candidate) = begin_on(&runtime, "candidate");
    candidate
        .push_batch(
            batch_create("candidate-only"),
            AllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let proposal = candidate
        .validate(&runtime, AllocationPolicy::SystemAllocation)
        .expect("candidate validation succeeds before sibling work");

    let counters_before_sibling = runtime.performance_access().counters();
    create_relation_in_partition_on_branch(
        &runtime,
        source,
        target,
        "sibling-relation",
        "sibling-relation",
        PartitionId::main(),
        BranchId("sibling".into()),
    );
    let counters_after_sibling = runtime.performance_access().counters();
    assert!(
        counters_after_sibling.relation_slots_touched_by_commit
            > counters_before_sibling.relation_slots_touched_by_commit,
        "the intervening sibling commit must touch a relation slot"
    );

    let candidate = runtime
        .prepare_validated_proposal(proposal)
        .expect("sibling advancement permits candidate revalidation");
    let exact_records = candidate
        .with_prepared_changed_records(|records| records.to_vec())
        .expect("the prepared owner retains its exact records");
    assert_eq!(
        candidate.prepared_changed_record_count(),
        Some(exact_records.len())
    );
    let committed = runtime
        .publish_prepared_candidate(candidate)
        .expect("prepared candidate publishes through its owner");
    assert_eq!(committed.changed_records, exact_records);

    assert_eq!(
        committed
            .complexity_delta()
            .relation_slots_touched_by_commit,
        0,
        "candidate accounting must exclude the sibling relation-slot work"
    );
    assert_eq!(
        committed.complexity_delta().commit_topology_flags,
        crate::transactions::data::CommitTopology::FlatEntityBatch.mask(),
        "publication must report only the candidate's exact flat topology"
    );
    assert_eq!(committed.complexity_delta().partitions_touched_by_commit, 1);
    assert_eq!(
        committed.complexity_delta().entity_slots_touched_by_commit,
        1
    );
}

#[test]
fn stale_transaction_reads_remain_on_the_admitted_root_and_commit_has_no_effect() {
    let runtime = runtime_with_test_schema();
    let entity = create_entity(&runtime, "basis-value");
    let (_, mut transaction) = begin_on(&runtime, "main");

    let before = transaction
        .read_entity(entity)
        .expect("basis read projects");
    assert_eq!(
        before.base().and_then(read_entity_name),
        Some("basis-value".into())
    );
    update_entity(&runtime, entity, "current-value");
    let after_movement = transaction
        .read_entity(entity)
        .expect("detached basis read projects");
    assert_eq!(
        after_movement.base().and_then(read_entity_name),
        Some("basis-value".into())
    );

    transaction
        .push_batch(
            update_batch(entity, "stale-write"),
            AllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    assert_eq!(
        transaction
            .read_entity(entity)
            .expect("staged field update projects")
            .effective()
            .and_then(read_entity_name),
        Some("stale-write".into())
    );
    let commit_count = runtime.history().immutable_commit_count();
    let symbols_before = runtime.services.symbols.clone();
    let symbol_table_before = runtime.config().identity.symbol_table.clone();
    let branch_cells_before = runtime.history().branch_cells_snapshot();
    let catalog_before = runtime.history().commit_envelopes_snapshot();
    let reference_cost_before = runtime.phase4_reference_cost_counters();
    let complexity_before = runtime.performance_access().counters();
    let error = transaction
        .commit(&runtime, AllocationPolicy::SystemAllocation)
        .expect_err("complete-reference movement stales the old basis");
    assert!(matches!(
        error,
        crate::facade::transactions::TransactionCommitError::Conflict { error, .. }
            if matches!(
                error.class,
                crate::facade::transactions::ConflictClass::StaleValidationBasis { .. }
            )
    ));
    assert_eq!(
        runtime.phase4_reference_cost_counters(),
        reference_cost_before
    );
    assert_eq!(runtime.history().immutable_commit_count(), commit_count);
    assert_eq!(runtime.services.symbols, symbols_before);
    assert_eq!(runtime.config().identity.symbol_table, symbol_table_before);
    assert_eq!(runtime.performance_access().counters(), complexity_before);
    assert_eq!(
        runtime.history().branch_cells_snapshot(),
        branch_cells_before
    );
    assert_eq!(
        runtime.history().commit_envelopes_snapshot(),
        catalog_before
    );
    let current_snapshot = runtime
        .publication()
        .latest_bundle()
        .expect("current publication exists")
        .snapshot
        .clone();
    let current = runtime
        .read_truth()
        .read_snapshot(&current_snapshot)
        .unwrap();
    assert_eq!(
        current.get_entity(entity).and_then(read_entity_name),
        Some("current-value".into())
    );
}

#[test]
fn stale_transaction_denies_before_interning_new_client_keys() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .client_key_symbol_policy(ClientKeySymbolPolicy::RequireInterned)
        .build();
    create_entity(&runtime, "stale-symbol-basis");
    let (_, mut transaction) = begin_on(&runtime, "main");
    create_entity(&runtime, "stale-symbol-advance");
    let symbols_before = runtime.services.symbols.clone();
    let symbol_table_before = runtime.config().identity.symbol_table.clone();
    let branch_cells_before = runtime.history().branch_cells_snapshot();
    let catalog_before = runtime.history().commit_envelopes_snapshot();
    let reference_cost_before = runtime.phase4_reference_cost_counters();
    let complexity_before = runtime.performance_access().counters();

    transaction
        .push_batch(
            batch_create("must-not-be-interned"),
            AllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let error = transaction
        .commit(&runtime, AllocationPolicy::SystemAllocation)
        .expect_err("stale currentness is checked before normalization");

    assert!(matches!(
        error,
        crate::facade::transactions::TransactionCommitError::Conflict { error, .. }
            if matches!(
                error.class,
                crate::facade::transactions::ConflictClass::StaleValidationBasis { .. }
            )
    ));
    assert_eq!(runtime.services.symbols, symbols_before);
    assert_eq!(runtime.config().identity.symbol_table, symbol_table_before);
    assert_eq!(
        runtime.phase4_reference_cost_counters(),
        reference_cost_before
    );
    assert_eq!(runtime.performance_access().counters(), complexity_before);
    assert_eq!(
        runtime.history().branch_cells_snapshot(),
        branch_cells_before
    );
    assert_eq!(
        runtime.history().commit_envelopes_snapshot(),
        catalog_before
    );
}

fn fork_from_main(runtime: &mut crate::facade::runtime::RelationalRuntime, branch: &str) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".into()))
        .expect("main has an exact fork source");
    runtime
        .fork_branch(BranchId(branch.into()), source)
        .expect("branch fork succeeds");
}

fn begin_on(
    runtime: &crate::facade::runtime::RelationalRuntime,
    branch: &str,
) -> (
    crate::facade::branch::AdmittedRelationalBranchBasis,
    crate::facade::mvcc::BranchBoundRelationalTransaction,
) {
    let identity = runtime
        .branch_identity(&BranchId(branch.into()))
        .expect("branch identity is owner-issued");
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("branch basis is owner-admitted");
    let transaction = runtime
        .begin_branch_transaction(
            &basis,
            crate::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("basis belongs to this runtime");
    (basis, transaction)
}

fn created_entity(name: &str) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: crate::facade::identity::PartitionId::main(),
        kind_id: crate::facade::identity::KindId(1),
        client_key: crate::facade::symbols::ClientKey::raw(name),
    }
}

fn update_batch(entity_id: crate::facade::identity::EntityId, name: &str) -> WorkerIntentBatch {
    WorkerIntentBatch::new(format!("update-{name}")).push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id,
            fields: name_field_patch(name),
        }),
    ))
}

fn create_intent(batch: &WorkerIntentBatch) -> CreateIntent {
    match &batch.intents[0] {
        MutationIntent::Create(intent) => intent.clone(),
        other => panic!("expected one create intent, got {other:?}"),
    }
}

mod savepoint_rollback;
