use crate::facade::identity::{KindId, PartitionId};
use crate::facade::transactions::{CreateIntent, MutationIntent, WorkerIntentBatch};
use crate::tests::support::*;
use std::sync::{Arc, Barrier};

#[test]
fn preparation_is_truth_effect_free_and_discard_releases_reservations() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "prepared-anchor");

    let branch_cells_before = runtime.history.branch_cells_snapshot();
    let envelopes_before = runtime.history().commit_envelopes_snapshot();
    let commit_count_before = runtime.history().immutable_commit_count();
    let patch_count_before = runtime.history.recorded_patch_position_count();
    let bundle_before = runtime.publication().latest_bundle();
    let diagnostics_before = runtime.publication().diagnostic_access().artifact_count();
    let durable_count_before = runtime.durability().durable_log().len();
    let candidate_cost_before = runtime
        .history
        .sharing_costs_for_branch(&BranchId("main".to_owned()));

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("prepared-discard"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let candidate = runtime
        .prepare_branch_transaction(
            execution,
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("all fallible preparation succeeds without publication");

    assert_eq!(runtime.history.branch_cells_snapshot(), branch_cells_before);
    assert_eq!(
        runtime.history().commit_envelopes_snapshot(),
        envelopes_before
    );
    assert_eq!(
        runtime.history().immutable_commit_count(),
        commit_count_before
    );
    assert_eq!(
        runtime.history.recorded_patch_position_count(),
        patch_count_before
    );
    assert_eq!(runtime.publication().latest_bundle(), bundle_before);
    assert_eq!(
        runtime.publication().diagnostic_access().artifact_count(),
        diagnostics_before
    );
    assert_eq!(
        runtime.durability().durable_log().len(),
        durable_count_before
    );
    assert_eq!(candidate.reservation_count(), 1);

    let discarded = runtime
        .discard_prepared_candidate(candidate)
        .expect("the owner consumes the candidate through explicit discard");
    assert_eq!(discarded.released_record_reservation_count(), 1);
    assert_eq!(discarded.branch(), &BranchId("main".to_owned()));
    assert_eq!(runtime.history.branch_cells_snapshot(), branch_cells_before);
    assert_eq!(
        runtime.history().commit_envelopes_snapshot(),
        envelopes_before
    );
    assert_eq!(
        runtime.history().immutable_commit_count(),
        commit_count_before
    );
    assert_eq!(
        runtime.history.recorded_patch_position_count(),
        patch_count_before
    );
    assert_eq!(runtime.publication().latest_bundle(), bundle_before);
    assert_eq!(
        runtime.publication().diagnostic_access().artifact_count(),
        diagnostics_before
    );
    assert_eq!(
        runtime.durability().durable_log().len(),
        durable_count_before
    );
    let candidate_cost_after = runtime
        .history
        .sharing_costs_for_branch(&BranchId("main".to_owned()));
    assert_eq!(
        candidate_cost_after.candidate_preparations,
        candidate_cost_before.candidate_preparations + 1
    );
    assert_eq!(
        candidate_cost_after.candidate_discards,
        candidate_cost_before.candidate_discards + 1
    );
    assert_eq!(
        candidate_cost_after.publication_attempts,
        candidate_cost_before.publication_attempts
    );
}

#[test]
fn prepared_root_materializes_exactly_the_declared_write_partitions() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity_in_partition(execution, &runtime, "main-anchor", PartitionId::main());
    create_entity_in_partition(execution, &runtime, "second-anchor", PartitionId(29));
    create_entity_in_partition(execution, &runtime, "untouched-anchor", PartitionId(41));

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            create_batch("main-write", PartitionId::main()),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .push_batch(
            create_batch("second-write", PartitionId(29)),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let declared_write_partition_count = transaction.footprint().write_partitions().len() as u64;
    let candidate = runtime
        .prepare_branch_transaction(
            execution,
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("candidate root construction succeeds");
    let (touched, reused) = candidate.materialization_counts();

    assert_eq!(touched, declared_write_partition_count);
    assert_eq!(touched, 2);
    assert_eq!(reused, 1, "the undeclared third partition must be reused");
    runtime
        .discard_prepared_candidate(candidate)
        .expect("materialization candidate remains discardable");
}

#[test]
fn prepared_candidate_is_sendable_across_one_worker_boundary() {
    fn assert_send<T: Send>() {}
    fn assert_clone_send_sync<T: Clone + Send + Sync>() {}
    assert_send::<crate::facade::mvcc::PreparedRelationalCommitCandidate>();
    assert_clone_send_sync::<crate::facade::mvcc::RelationalPublicationPort>();
}

fn create_batch(name: &str, partition_id: PartitionId) -> WorkerIntentBatch {
    WorkerIntentBatch::new(format!("batch-{name}")).push(MutationIntent::Create(
        CreateIntent::Entity(crate::transactions::data::EntitySpec {
            partition_id,
            kind_id: KindId(1),
            client_key: crate::symbols::data::ClientKey::raw(name),
            fields: entity_fields_for_runtime_name(name),
        }),
    ))
}

fn entity_fields_for_runtime_name(name: &str) -> crate::transactions::data::AspectFieldPatch {
    single_string_aspect_field_patch(aspect_key("name"), field_key("name"), name)
}

#[path = "prepared_candidates/publication_race.rs"]
mod publication_race;
