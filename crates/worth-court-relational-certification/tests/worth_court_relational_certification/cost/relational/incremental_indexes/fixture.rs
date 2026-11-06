pub(super) use crate::index_retirement::{release_test_commit_snapshot, runtime_with_test_schema};
use crate::leased_preparation::ordinary_context::support::{
    aspect_key, field_key, main_basis, string_field_patch,
};
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest, ExecutionWorkCeiling};
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId, RelationId},
    indexes::DerivedIndexMaintenanceBudget,
    mvcc::{
        BranchBoundRelationalTransaction, PreparedRelationalCommitCandidate,
        RelationalPublicationOutcome, RelationalTransactionIntent,
    },
    runtime::RelationalRuntime,
    symbols::ClientKey,
    transactions::{
        CommitResult, CreateIntent, EntityReference, EntitySpec, MutationIntent, RecordRef,
        RelationSpec, WorkerIntentBatch,
    },
};

pub(super) fn transaction(runtime: &RelationalRuntime) -> BranchBoundRelationalTransaction {
    runtime
        .begin_branch_transaction(
            &main_basis(runtime),
            RelationalTransactionIntent::ordinary(),
        )
        .unwrap()
}
pub(super) fn entities(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    count: usize,
) -> Vec<EntityId> {
    let mut transaction = transaction(runtime);
    let mut batch = WorkerIntentBatch::new("scale-records");
    for ordinal in 0..count {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: ClientKey::raw(format!("row-{ordinal}")),
            fields: string_field_patch(
                aspect_key("name"),
                field_key("name"),
                &format!("row-{ordinal}"),
            ),
        })));
    }
    transaction
        .push_batch(batch, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let committed = transaction.commit(execution, runtime).unwrap();
    let ids = committed
        .changed_records
        .iter()
        .filter_map(|record| match record {
            RecordRef::Entity(id) => Some(*id),
            _ => None,
        })
        .collect();
    release_test_commit_snapshot(runtime, &committed);
    ids
}
pub(super) fn relations(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    endpoints: &[(EntityId, EntityId)],
) {
    let mut transaction = transaction(runtime);
    let mut batch = WorkerIntentBatch::new("scale-relations");
    for (ordinal, (source, target)) in endpoints.iter().enumerate() {
        batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw(format!("edge-{ordinal}")),
                source: EntityReference::Existing(*source),
                target: EntityReference::Existing(*target),
                fields: string_field_patch(
                    aspect_key("label"),
                    field_key("label"),
                    &format!("edge-{ordinal}"),
                ),
            },
        )));
    }
    transaction
        .push_batch(batch, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let committed = transaction.commit(execution, runtime).unwrap();
    release_test_commit_snapshot(runtime, &committed);
}
pub(super) fn relation_between(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: EntityId,
    target: EntityId,
) -> RelationId {
    let basis = main_basis(runtime);
    let snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let view = runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .unwrap()
        .unwrap();
    let relation = view
        .relations()
        .iter()
        .filter(|relation| relation.source == source && relation.target == target)
        .min_by_key(|relation| relation.relation_id)
        .unwrap()
        .relation_id;
    runtime.snapshots().release_snapshot(&snapshot).unwrap();
    relation
}
pub(super) fn budget() -> DerivedIndexMaintenanceBudget {
    DerivedIndexMaintenanceBudget {
        maximum_work_units: None,
        maximum_cold_record_slots: Some(0),
        maximum_derived_rows: None,
    }
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Work {
    units: u64,
    payloads: u64,
    inventory: u64,
    records: u64,
    adjacency: u64,
}
pub(super) fn prepare(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    transaction: BranchBoundRelationalTransaction,
) -> (PreparedRelationalCommitCandidate, Work) {
    let mut candidate = runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    let before = runtime.index_access().generation_selection_counters();
    let (prepared, report) = execution
        .run(ExecutionWorkCeiling::new(None), |_| {
            runtime
                .index_authority()
                .prepare_routed_indexes_for_candidate(&mut candidate, budget(), execution)
        })
        .unwrap();
    let maintained = prepared.unwrap();
    let after = runtime.index_access().generation_selection_counters();
    (
        candidate,
        Work {
            units: report.charged_work(),
            payloads: (after.generation_payload_reads - before.generation_payload_reads) as u64,
            inventory: (after.history_inventory_entries - before.history_inventory_entries) as u64,
            records: maintained.record_reads as u64,
            adjacency: maintained.adjacency_work_units as u64,
        },
    )
}
pub(super) fn publish(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    candidate: PreparedRelationalCommitCandidate,
) -> CommitResult {
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("funded candidate publishes")
    };
    runtime
        .settlement_port()
        .settle_performed_publication(performed)
        .unwrap()
}
pub(super) fn flat(axis: &str, small: Work, large: Work) {
    eprintln!("{axis}: {small:?} -> {large:?}");
    assert!(small.units > 0, "the index phase must be metered");
    for (component, small, large) in [
        ("charged units", small.units, large.units),
        ("generation payloads", small.payloads, large.payloads),
        ("history inventory", small.inventory, large.inventory),
        ("record reads", small.records, large.records),
        ("adjacency work", small.adjacency, large.adjacency),
    ] {
        assert!(
            u128::from(large.max(1)) * 2 < u128::from(small.max(1)) * 3,
            "{axis}/{component}: index work grew {small} -> {large}"
        );
    }
}
