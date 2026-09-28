//! Relational fixtures for the adapter tests, built on public Relational API
//! only, so they move with the adapter.

mod publication;
mod schema;

use std::collections::BTreeMap;

use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, InternedString};

use crate::facade::history::BranchId;
use crate::facade::identity::{EntityId, KindId, PartitionId, RelationId};
use crate::facade::mvcc::{BranchBoundRelationalTransaction, RelationalTransactionIntent};
use crate::facade::runtime::RelationalRuntime;
use crate::facade::symbols::ClientKey;
use crate::facade::transactions::{
    planned_single_field_locator, AspectFieldPatch, CommitResult, CreateIntent,
    EntityMutationIntent, EntityReference, EntitySpec, MutationIntent, RecordRef, RelationSpec,
    ReplaceEntityIntent, WorkerIntentBatch,
};

pub(crate) use publication::{
    exact_change, published_patch, widened_change, WireOperation, WireValue,
};
pub(crate) use schema::{
    entity_summary_struct_aspect, runtime_with_declared_aspect_schema, AspectSchemaFixture,
};

const ENTITY_KIND: KindId = KindId(1);
const RELATION_KIND: KindId = KindId(2);

pub(crate) fn aspect_key(name: &str) -> AspectKey {
    AspectKey::new(name).unwrap()
}

pub(crate) fn field_key(name: &str) -> FieldKey {
    FieldKey::new(name).expect("test field names must be foundational field keys")
}

pub(crate) fn single_string_aspect_field_patch(
    aspect_key: AspectKey,
    field: FieldKey,
    value: &str,
) -> AspectFieldPatch {
    string_aspect_field_patch([(aspect_key, field, value)])
}

pub(crate) fn string_aspect_field_patch<'a>(
    fields: impl IntoIterator<Item = (AspectKey, FieldKey, &'a str)>,
) -> AspectFieldPatch {
    AspectFieldPatch::from(
        fields
            .into_iter()
            .map(|(aspect_key, field, value)| {
                (
                    planned_single_field_locator(aspect_key, field),
                    AspectValue::String(InternedString::Raw(value.to_owned())),
                )
            })
            .collect::<BTreeMap<_, _>>(),
    )
}

pub(crate) fn test_owner_begin_transaction_for_main(
    runtime: &RelationalRuntime,
) -> BranchBoundRelationalTransaction {
    test_owner_begin_transaction_for_branch(runtime, BranchId("main".to_owned()))
}

/// Begin a transaction on a registered branch through its admitted basis.
pub(crate) fn test_owner_begin_transaction_for_branch(
    runtime: &RelationalRuntime,
    branch_id: BranchId,
) -> BranchBoundRelationalTransaction {
    let identity = runtime
        .branch_identity(&branch_id)
        .unwrap_or_else(|error| panic!("test branch must be owner-registered: {error:?}"));
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("registered test branch remains owner-admissible");
    runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .expect("branch basis belongs to its issuing runtime")
}

pub(crate) fn create_entity(runtime: &RelationalRuntime, name: &str) -> EntityId {
    let outcome = create_entity_outcome(runtime, name);
    let entity = changed_entities(&outcome)[0];
    release_test_commit_snapshot(runtime, &outcome);
    entity
}

pub(crate) fn create_entity_in_partition(
    runtime: &RelationalRuntime,
    name: &str,
    partition_id: PartitionId,
) -> EntityId {
    let outcome = commit_named_entity(runtime, name, partition_id, BranchId("main".to_owned()));
    let entity = changed_entities(&outcome)[0];
    release_test_commit_snapshot(runtime, &outcome);
    entity
}

pub(crate) fn create_entity_outcome(runtime: &RelationalRuntime, name: &str) -> CommitResult {
    create_entity_outcome_on_branch(runtime, name, BranchId("main".to_owned()))
}

pub(crate) fn create_entity_outcome_on_branch(
    runtime: &RelationalRuntime,
    name: &str,
    branch_id: BranchId,
) -> CommitResult {
    commit_named_entity(runtime, name, PartitionId::main(), branch_id)
}

fn commit_named_entity(
    runtime: &RelationalRuntime,
    name: &str,
    partition_id: PartitionId,
    branch_id: BranchId,
) -> CommitResult {
    let mut transaction = test_owner_begin_transaction_for_branch(runtime, branch_id);
    transaction
        .push_batch(
            WorkerIntentBatch::new(format!("batch-{name}")).push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id,
                    kind_id: ENTITY_KIND,
                    client_key: ClientKey::raw(name),
                    fields: single_string_aspect_field_patch(
                        aspect_key("name"),
                        field_key("name"),
                        name,
                    ),
                }),
            )),
        )
        .unwrap();
    transaction.commit(runtime).unwrap()
}

/// Fork `new_branch` from the current root of `from_branch`.
pub(crate) fn fork_branch(
    runtime: &RelationalRuntime,
    new_branch: BranchId,
    from_branch: &BranchId,
) {
    let (_, source) = runtime
        .observe_fork_source(from_branch)
        .expect("fork source branch is owner-admissible");
    runtime
        .fork_branch(new_branch, source)
        .expect("fork target branch is new");
}

/// Replace `entity` on `branch` with a new named entity that continues its
/// lineage.
pub(crate) fn replace_entity_on_branch(
    runtime: &RelationalRuntime,
    entity: EntityId,
    replacement: &str,
    branch: BranchId,
) -> CommitResult {
    let mut transaction = test_owner_begin_transaction_for_branch(runtime, branch);
    transaction
        .push_batch(
            WorkerIntentBatch::new(replacement).push(MutationIntent::Entity(
                EntityMutationIntent::Replace(ReplaceEntityIntent {
                    entity_id: entity,
                    replacement: EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: ENTITY_KIND,
                        client_key: ClientKey::raw(replacement),
                        fields: single_string_aspect_field_patch(
                            aspect_key("name"),
                            field_key("name"),
                            replacement,
                        ),
                    },
                }),
            )),
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .commit(runtime)
        .expect("replacement should commit")
}

/// The entity a replacement commit created in place of `original`.
pub(crate) fn replacement_successor(outcome: &CommitResult, original: EntityId) -> EntityId {
    let successors = changed_entities(outcome)
        .into_iter()
        .filter(|entity| *entity != original)
        .collect::<Vec<_>>();
    assert_eq!(successors.len(), 1, "a replacement creates one successor");
    successors[0]
}

/// Relate `source` to `target` on main with a `label` field.
pub(crate) fn create_relation_outcome(
    runtime: &RelationalRuntime,
    source: EntityId,
    target: EntityId,
    client_key: &str,
) -> CommitResult {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("relation").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: RELATION_KIND,
                    client_key: ClientKey::raw(client_key),
                    source: EntityReference::Existing(source),
                    target: EntityReference::Existing(target),
                    fields: single_string_aspect_field_patch(
                        aspect_key("label"),
                        field_key("label"),
                        client_key,
                    ),
                }),
            )),
        )
        .unwrap();
    transaction.commit(runtime).unwrap()
}

pub(crate) fn changed_relations(outcome: &CommitResult) -> Vec<RelationId> {
    outcome
        .changed_records
        .iter()
        .filter_map(|record| match record {
            RecordRef::Relation(relation_id) => Some(*relation_id),
            RecordRef::Entity(_) => None,
        })
        .collect()
}

pub(crate) fn changed_entities(outcome: &CommitResult) -> Vec<EntityId> {
    outcome
        .changed_records
        .iter()
        .filter_map(|record| match record {
            RecordRef::Entity(entity_id) => Some(*entity_id),
            RecordRef::Relation(_) => None,
        })
        .collect()
}

pub(crate) fn release_test_commit_snapshot(runtime: &RelationalRuntime, outcome: &CommitResult) {
    runtime
        .snapshots()
        .release_snapshot(&outcome.snapshot)
        .expect("test helper releases the published snapshot it does not return");
}
