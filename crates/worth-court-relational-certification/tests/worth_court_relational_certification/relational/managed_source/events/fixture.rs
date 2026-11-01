//! Authored generic physical events and actual transaction observations.
use super::*;
pub(super) fn link(kind: u32, a: EntityId, b: EntityId, label: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(kind),
        client_key: ClientKey::raw(label),
        source: EntityReference::Existing(a),
        target: EntityReference::Existing(b),
        fields: Default::default(),
    }))
}
pub(super) fn relation(
    born: &CommitResult,
    kind: u32,
    a: EntityId,
    b: EntityId,
    label: &str,
) -> RelationId {
    born.created_relation(&CreatedRelationRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(kind),
        client_key: ClientKey::raw(label),
        source: EntityReference::Existing(a),
        target: EntityReference::Existing(b),
    })
    .unwrap()
}
pub(super) struct World {
    pub events: [EntityId; 5],
    pub rows: [EntityId; 3],
}
pub(super) fn seed(runtime: &RelationalRuntime, execution: &ExecutionRequest<'_, '_>) -> World {
    let mut batch = WorkerIntentBatch::new("event-chain-birth");
    for (kind, name) in [
        (1, "a"),
        (1, "b"),
        (1, "c"),
        (1, "d"),
        (1, "u"),
        (4, "x"),
        (4, "y"),
        (4, "z"),
    ] {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(kind),
            client_key: ClientKey::raw(name),
            fields: Default::default(),
        })));
    }
    let born = commit(runtime, execution, batch);
    let get = |kind, name| born.created_entity(&created(kind, name)).unwrap();
    World {
        events: [
            get(1, "a"),
            get(1, "b"),
            get(1, "c"),
            get(1, "d"),
            get(1, "u"),
        ],
        rows: [get(4, "x"), get(4, "y"), get(4, "z")],
    }
}
pub(super) fn expect(
    control: &Control,
    court: &owner::Court,
    world: &World,
    before: [&[usize]; 5],
    after: [&[usize]; 5],
    affected: &[usize],
) {
    let values = |indices: &[usize]| {
        let mut values = indices.iter().map(|i| world.rows[*i]).collect::<Vec<_>>();
        values.sort();
        values
    };
    *control.expected.lock().unwrap() = world
        .events
        .iter()
        .enumerate()
        .map(|(i, at)| (*at, values(after[i])))
        .collect();
    *court.expected.lock().unwrap() = affected
        .iter()
        .map(|i| (world.events[*i], (values(before[*i]), values(after[*i]))))
        .collect();
    court
        .event_cohorts
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .clear();
    court.staged.lock().unwrap().clear();
    court.observed.lock().unwrap().clear();
}
#[track_caller]
pub(super) fn assert_scopes(court: &owner::Court, world: &World, affected: &[usize]) {
    let wanted = affected
        .iter()
        .map(|i| world.events[*i])
        .collect::<BTreeSet<_>>();
    assert_eq!(
        *court.staged.lock().unwrap(),
        wanted,
        "Native Source staging"
    );
    assert_eq!(
        *court.observed.lock().unwrap(),
        wanted,
        "dependent rule invocation"
    );
}
pub(super) fn delete(relation: RelationId) -> MutationIntent {
    MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
        relation_id: relation,
    }))
}
pub(super) fn try_commit(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    batch: WorkerIntentBatch,
) -> Result<CommitResult, TransactionCommitError> {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut tx = runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    tx.push_batch(
        batch,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    tx.commit(execution, runtime)
}

pub(super) fn assert_changed(court: &owner::Court, event: EntityId, rows: &[EntityId]) {
    let cohorts = court.event_cohorts.lock().unwrap();
    let actual = cohorts
        .as_ref()
        .unwrap()
        .get(&event)
        .expect("actual staged scope cohort");
    for row in rows {
        assert!(
            actual.contains(row),
            "complete changed rows retain removed/continuing member {row:?}"
        );
    }
}
