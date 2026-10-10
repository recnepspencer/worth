use crate::facade::branch::{
    RelationalBranchBasisDenial, RelationalBranchDeleteDenial, RelationalForkDenial,
};
use crate::facade::history::BranchId;
use crate::facade::mvcc::{
    RelationalBranchTransactionAdmissionDenial, RelationalPublicationDeferred,
    RelationalTransactionIntent,
};
use crate::facade::transactions::TransactionCommitError;
use crate::tests::support::*;

#[test]
fn live_obligation_capacity_denies_each_public_acquisition_without_residue() {
    let mut observation_runtime = runtime_with_test_schema();
    create_entity(&observation_runtime, "observation-capacity-anchor");
    let observation_identity = observation_runtime.main_branch_identity();
    observation_runtime.set_retention_capacity_for_test(1, 8);
    assert_eq!(
        observation_runtime
            .observe_branch(&observation_identity)
            .unwrap_err(),
        RelationalBranchBasisDenial::RetentionCapacityExhausted,
    );
    observation_runtime.set_retention_capacity_for_test(2, 8);
    drop(
        observation_runtime
            .observe_branch(&observation_identity)
            .unwrap(),
    );

    let mut transaction_runtime = runtime_with_test_schema();
    create_entity(&transaction_runtime, "transaction-capacity-anchor");
    transaction_runtime.set_retention_capacity_for_test(2, 8);
    let transaction_identity = transaction_runtime.main_branch_identity();
    let (_, transaction_basis) = transaction_runtime
        .observe_branch(&transaction_identity)
        .unwrap();
    assert_eq!(
        transaction_runtime
            .begin_branch_transaction(&transaction_basis, RelationalTransactionIntent::ordinary(),)
            .unwrap_err(),
        RelationalBranchTransactionAdmissionDenial::RetentionCapacityExhausted,
    );
    drop(transaction_basis);
    transaction_runtime.set_retention_capacity_for_test(3, 8);
    let (_, replacement_basis) = transaction_runtime
        .observe_branch(&transaction_identity)
        .unwrap();
    drop(
        transaction_runtime
            .begin_branch_transaction(&replacement_basis, RelationalTransactionIntent::ordinary())
            .unwrap(),
    );

    let mut candidate_runtime = runtime_with_test_schema();
    create_entity(&candidate_runtime, "candidate-capacity-anchor");
    candidate_runtime.set_retention_capacity_for_test(3, 8);
    let candidate_identity = candidate_runtime.main_branch_identity();
    let (_, candidate_basis) = candidate_runtime
        .observe_branch(&candidate_identity)
        .unwrap();
    let mut transaction = candidate_runtime
        .begin_branch_transaction(&candidate_basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            batch_create("candidate-capacity-write"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate_reference_before = candidate_runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .unwrap();
    assert!(matches!(
        candidate_runtime.prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        ),
        Err(TransactionCommitError::PublicationDeferred {
            deferred: RelationalPublicationDeferred::RetentionBackpressure,
            ..
        })
    ));
    assert_eq!(
        candidate_runtime
            .branch_reference_state(&BranchId("main".to_owned()))
            .unwrap(),
        candidate_reference_before,
    );
    drop(candidate_basis);
    candidate_runtime.set_retention_capacity_for_test(4, 8);
    let (_, replacement_basis) = candidate_runtime
        .observe_branch(&candidate_identity)
        .unwrap();
    let replacement = candidate_runtime
        .begin_branch_transaction(&replacement_basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let replacement = candidate_runtime
        .prepare_branch_transaction(
            replacement,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    candidate_runtime
        .discard_prepared_candidate(replacement)
        .unwrap();
}

#[test]
fn pending_delete_reserves_its_retired_name_before_other_deletions_consume_capacity() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "pending-delete-capacity-anchor");
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .unwrap();
    let branch_id = BranchId("reserved-pending-delete".to_owned());
    runtime.fork_branch(branch_id.clone(), source).unwrap();
    let identity = runtime.branch_identity(&branch_id).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();

    let waiting = runtime.delete_branch(&identity).unwrap();
    assert_eq!(waiting.waiting().unwrap().active_operation_count(), 1);
    runtime.fill_retired_branch_name_capacity_for_test();
    drop(transaction);
    drop(basis);

    assert!(runtime
        .delete_branch(&identity)
        .unwrap()
        .deleted()
        .is_some());
}

#[test]
fn head_and_retirement_capacity_fail_before_reference_movement() {
    let mut fork_runtime = runtime_with_test_schema();
    create_entity(&fork_runtime, "fork-capacity-anchor");
    fork_runtime.set_retention_capacity_for_test(1, 8);
    let (_, source) = fork_runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .unwrap();
    assert_eq!(
        fork_runtime
            .fork_branch(BranchId("capacity-fork".to_owned()), source)
            .unwrap_err(),
        RelationalForkDenial::RetentionCapacityExhausted,
    );
    assert!(fork_runtime
        .branch_identity(&BranchId("capacity-fork".to_owned()))
        .is_err());

    let mut publication_runtime = runtime_with_test_schema();
    create_entity(&publication_runtime, "publication-capacity-anchor");
    publication_runtime.set_retention_capacity_for_test(4, 0);
    let identity = publication_runtime.main_branch_identity();
    let (_, basis) = publication_runtime.observe_branch(&identity).unwrap();
    let reference_before = publication_runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .unwrap();
    let mut transaction = publication_runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            batch_create("publication-capacity-write"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = publication_runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        publication_runtime
            .publication_port()
            .compare_and_publish(candidate),
        crate::mvcc::RelationalPublicationOutcome::Deferred(
            RelationalPublicationDeferred::RetentionBackpressure
        )
    ));
    assert_eq!(
        publication_runtime
            .branch_reference_state(&BranchId("main".to_owned()))
            .unwrap(),
        reference_before,
    );
    assert_eq!(
        publication_runtime
            .history
            .pending_canonical_publication_route_count(),
        0
    );
    drop(basis);
    publication_runtime.set_retention_capacity_for_test(4, 8);
    let (_, replacement_basis) = publication_runtime.observe_branch(&identity).unwrap();
    let replacement = publication_runtime
        .begin_branch_transaction(&replacement_basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let replacement = publication_runtime
        .prepare_branch_transaction(
            replacement,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    publication_runtime
        .discard_prepared_candidate(replacement)
        .unwrap();

    let mut deletion_runtime = runtime_with_test_schema();
    create_entity(&deletion_runtime, "deletion-capacity-anchor");
    let (_, source) = deletion_runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .unwrap();
    deletion_runtime
        .fork_branch(BranchId("capacity-delete".to_owned()), source)
        .unwrap();
    let deletion_identity = deletion_runtime
        .branch_identity(&BranchId("capacity-delete".to_owned()))
        .unwrap();
    deletion_runtime.set_retention_capacity_for_test(8, 0);
    assert_eq!(
        deletion_runtime
            .delete_branch(&deletion_identity)
            .unwrap_err(),
        RelationalBranchDeleteDenial::RetentionBackpressure,
    );
    deletion_runtime
        .observe_branch(&deletion_identity)
        .expect("retirement backpressure cannot leave the branch deleting");
}

#[test]
fn capacity_consumed_after_preparation_defers_next_basis_before_movement() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "post-candidate-capacity-anchor");
    runtime.set_retention_capacity_for_test(4, 8);
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let reference_before = runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            batch_create("post-candidate-capacity-write"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let external = runtime.retain_component_basis(&basis).unwrap();

    assert!(matches!(
        runtime.publication_port().compare_and_publish(candidate),
        crate::mvcc::RelationalPublicationOutcome::Deferred(
            RelationalPublicationDeferred::RetentionBackpressure
        )
    ));
    assert_eq!(
        runtime
            .branch_reference_state(&BranchId("main".to_owned()))
            .unwrap(),
        reference_before
    );
    runtime.release_component_basis(external).unwrap();
    drop(basis);
    let (_, replacement) = runtime.observe_branch(&identity).unwrap();
    drop(replacement);
}

#[test]
fn prepared_root_cost_is_observational_and_system_publication_remains_current() {
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::AiWorkflow)
        .schema_registry(test_schema_registry())
        .publication(crate::facade::config::PublicationConfig {
            coherent_publication_required: true,
            max_patch_records_per_commit: 4_096,
            max_published_snapshot_handles: 8,
            max_active_snapshot_handles: 8,
            max_transaction_savepoints: 8,
            max_prepared_candidates: 8,
            candidate_max_lifetime_millis: 30_000,
        })
        .build();
    let before = test_owner_main_basis(&runtime).unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&before, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            batch_create("observed-root-cost"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert_eq!(
        test_owner_main_basis(&runtime).unwrap().descriptor(),
        before.descriptor()
    );
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) =
        runtime.publication_port().compare_and_publish(candidate)
    else {
        panic!("a prepared system-allocated root must publish");
    };
    let cost = performed.next_basis().inner.root.publication_cost();
    assert!(cost.new_authoritative_bytes > 0);
    assert_eq!(cost.touched_regions, 1);
    assert_eq!(
        test_owner_main_basis(&runtime).unwrap().descriptor(),
        performed.next_basis().descriptor(),
    );
    let committed = runtime.settle_performed_publication(performed).unwrap();
    let entities = changed_entities(&committed);
    assert_eq!(entities.len(), 1);
    let records = runtime
        .read_truth()
        .project_historical_version(committed.commit.version_id)
        .all_authoritative_entity_records();
    let record = records
        .iter()
        .find(|record| record.entity_id == entities[0])
        .unwrap();
    assert_eq!(
        read_entity_name(record),
        Some("observed-root-cost".to_owned())
    );
    assert_eq!(
        runtime.history.pending_canonical_publication_route_count(),
        0
    );
    release_test_commit_snapshot(&runtime, &committed);
}
