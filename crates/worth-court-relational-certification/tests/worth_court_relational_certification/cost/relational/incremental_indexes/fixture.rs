pub(super) use crate::index_retirement::{release_test_commit_snapshot, runtime_with_test_schema};
use crate::leased_preparation::ordinary_context::support::{
    aspect_key, field_key, main_basis, string_field_patch,
};
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId, RelationId},
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
    let before = runtime.index_access().generation_selection_counters();
    let (candidate, observation) =
        worth_relational::facade::inspection::observe_maintenance_work(32, || {
            runtime
                .prepare_branch_transaction(execution, transaction)
                .unwrap()
        });
    let maintained = candidate
        .prepared_index_maintenance_work()
        .expect("native preparation carries its actual receipt");
    assert_eq!(observation.omitted_phases, 0);
    let phases: Vec<_> = observation
        .phases
        .iter()
        .filter(|row| row.phase == "index_maintenance")
        .collect();
    assert!(
        !phases.is_empty(),
        "the actual pipeline enters the native index phase"
    );
    let units = phases
        .into_iter()
        .map(|row| {
            assert!(row.accounted, "phase work uses the original request's bill");
            row.charged_work
        })
        .sum();
    let after = runtime.index_access().generation_selection_counters();
    (
        candidate,
        Work {
            units,
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
        worth_testkit_relational::maintenance::assert_change_bounded(
            &format!("index_maintenance/{component}"),
            axis,
            small.max(1),
            large.max(1),
        );
    }
}
