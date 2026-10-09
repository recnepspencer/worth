//! Native branch fixture authorship under explicit uncharged physical selection.
use super::*;

pub(super) fn create_entity_on_branch(
    runtime: &mut RelationalRuntime,
    name: &str,
    branch_id: BranchId,
) -> EntityId {
    let mut txn = begin_branch_transaction(runtime, &branch_id);
    txn.push_batch(
        WorkerIntentBatch::new(format!("create-{name}")).push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw(name),
                fields: single_native_string_aspect_field_patch("name", "name", name)
                    .expect("entity name aspect patch"),
            }),
        )),
        Allocation::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(runtime, Allocation::SystemAllocation)
        .expect("entity create should commit");
    let entity = changed_entities(&outcome)[0];
    release_commit_snapshot(runtime, &outcome);
    entity
}

pub(super) fn update_entity_on_branch(
    runtime: &mut RelationalRuntime,
    entity_id: EntityId,
    name: &str,
    branch_id: BranchId,
) {
    let mut txn = begin_branch_transaction(runtime, &branch_id);
    txn.push_batch(
        WorkerIntentBatch::new("update-entity").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id,
                fields: single_native_string_aspect_field_patch("name", "name", name)
                    .expect("entity name aspect patch"),
            }),
        )),
        Allocation::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(runtime, Allocation::SystemAllocation)
        .expect("entity update should commit");
    release_commit_snapshot(runtime, &outcome);
}

pub(super) fn delete_entity_on_branch(
    runtime: &mut RelationalRuntime,
    entity_id: EntityId,
    branch_id: BranchId,
) {
    let mut txn = begin_branch_transaction(runtime, &branch_id);
    txn.push_batch(
        WorkerIntentBatch::new("delete-entity").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id }),
        )),
        Allocation::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(runtime, Allocation::SystemAllocation)
        .expect("entity delete should commit");
    release_commit_snapshot(runtime, &outcome);
}

pub(super) fn create_relation_on_branch(
    runtime: &mut RelationalRuntime,
    source: EntityId,
    target: EntityId,
    client_key: &str,
    label: &str,
    branch_id: BranchId,
) -> RelationId {
    let mut txn = begin_branch_transaction(runtime, &branch_id);
    txn.push_batch(
        WorkerIntentBatch::new("create-relation").push(MutationIntent::Create(
            CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw(client_key),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(target),
                fields: single_native_string_aspect_field_patch("label", "label", label)
                    .expect("relation label aspect patch"),
            }),
        )),
        Allocation::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(runtime, Allocation::SystemAllocation)
        .expect("relation create should commit");
    let relation = changed_relations(&outcome)[0];
    release_commit_snapshot(runtime, &outcome);
    relation
}

pub(super) fn delete_relation_on_branch(
    runtime: &mut RelationalRuntime,
    relation_id: RelationId,
    branch_id: BranchId,
) {
    let mut txn = begin_branch_transaction(runtime, &branch_id);
    txn.push_batch(
        WorkerIntentBatch::new("delete-relation").push(MutationIntent::Relation(
            RelationMutationIntent::Delete(DeleteRelationIntent { relation_id }),
        )),
        Allocation::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(runtime, Allocation::SystemAllocation)
        .expect("relation delete should commit");
    release_commit_snapshot(runtime, &outcome);
}

fn release_commit_snapshot(runtime: &mut RelationalRuntime, outcome: &CommitResult) {
    runtime
        .snapshots()
        .release_snapshot(&outcome.snapshot)
        .expect("merge inspection fixture releases its exact commit snapshot");
}

fn begin_branch_transaction(
    runtime: &RelationalRuntime,
    branch_id: &BranchId,
) -> worth_relational::facade::mvcc::BranchBoundRelationalTransaction {
    let identity = runtime.branch_identity(branch_id).expect("branch identity");
    let context = runtime
        .admit_branch_basis(&identity)
        .expect("branch context");
    runtime
        .begin_branch_transaction(
            &context,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("owner-admitted branch basis")
}

fn changed_entities(outcome: &CommitResult) -> Vec<EntityId> {
    outcome
        .changed_records
        .iter()
        .filter_map(|record| match record {
            RecordRef::Entity(entity_id) => Some(*entity_id),
            RecordRef::Relation(_) => None,
        })
        .collect()
}
fn changed_relations(outcome: &CommitResult) -> Vec<RelationId> {
    outcome
        .changed_records
        .iter()
        .filter_map(|record| match record {
            RecordRef::Relation(relation_id) => Some(*relation_id),
            RecordRef::Entity(_) => None,
        })
        .collect()
}
