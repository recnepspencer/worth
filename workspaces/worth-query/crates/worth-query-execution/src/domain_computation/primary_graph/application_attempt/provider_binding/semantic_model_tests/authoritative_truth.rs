use std::collections::BTreeMap;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::identity::{EntityId, KindId, PartitionId, RelationId};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, CreatedEntityRef, EntityReference, EntitySpec, MutationIntent,
    RecordRef, RelationSpec, WorkerIntentBatch,
};

use crate::domain_computation::primary_graph::application_attempt::observation::{
    exact_relations, observe_field_value,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;

mod schema;

pub(super) struct CommittedObservationWorld {
    pub(super) update_entity: EntityId,
    pub(super) deleted_entity: EntityId,
    pub(super) relation_from: EntityId,
    pub(super) relation_to: EntityId,
    pub(super) deleted_relation: RelationId,
    pub(super) facts: Vec<WorthQueryApplicationObservedFact>,
}

pub(super) fn compile_committed_observations(
    alpha: &AspectFieldLocator,
    beta: &AspectFieldLocator,
) -> CommittedObservationWorld {
    let mut runtime = schema::fixture_runtime();
    let entities = commit_entities(&mut runtime, alpha, beta);
    let decoy_relation = commit_relation(
        &mut runtime,
        RelationSeed {
            batch: "provider-effect-decoy-relation",
            key: "decoy-edge",
            from: entities.decoy_from,
            to: entities.decoy_to,
        },
    );
    let relation_commit = commit_relation(
        &mut runtime,
        RelationSeed {
            batch: "provider-effect-observed-relation",
            key: "observed-edge",
            from: entities.from,
            to: entities.to,
        },
    );
    let relation = created_relation(&relation_commit);
    let axes = ObservedAxes {
        update: entities.update,
        decoy_update: entities.decoy_update,
        deleted: entities.deleted,
        decoy_deleted: entities.decoy_deleted,
        from: entities.from,
        to: entities.to,
        decoy_from: entities.decoy_from,
        decoy_to: entities.decoy_to,
        relation,
        decoy_relation: created_relation(&decoy_relation),
    };
    let locators = ObservedLocators { alpha, beta };
    let facts = observed_facts(&runtime, &relation_commit.snapshot, axes, locators);
    crate::relational_snapshot_release::release_query_snapshot(
        &mut runtime,
        &relation_commit.snapshot,
    );
    crate::relational_snapshot_release::release_query_snapshot(
        &mut runtime,
        &decoy_relation.snapshot,
    );
    CommittedObservationWorld {
        update_entity: entities.update,
        deleted_entity: entities.deleted,
        relation_from: entities.from,
        relation_to: entities.to,
        deleted_relation: relation,
        facts,
    }
}

fn created_relation(
    committed: &worth_relational::facade::transactions::CommitResult,
) -> RelationId {
    committed
        .changed_records
        .iter()
        .find_map(|record| match record {
            RecordRef::Relation(id) => Some(*id),
            RecordRef::Entity(_) => None,
        })
        .expect("authoritative relation commit issued an identity")
}

fn commit_entities(
    runtime: &mut RelationalRuntime,
    alpha: &AspectFieldLocator,
    beta: &AspectFieldLocator,
) -> CommittedEntities {
    let update = created(11, "observed-update");
    let decoy_update = created(11, "decoy-update");
    let deleted = created(12, "observed-delete");
    let decoy_deleted = created(12, "decoy-delete");
    let from = created(13, "relation-from");
    let to = created(14, "relation-to");
    let decoy_from = created(13, "decoy-relation-from");
    let decoy_to = created(14, "decoy-relation-to");
    let committed = commit(
        runtime,
        "provider-effect-observed-entities",
        [
            create_entity(
                &update,
                super::world::values([(alpha, "one-before"), (beta, "two-before")]),
            ),
            create_entity(
                &decoy_update,
                super::world::values([(alpha, "decoy-one"), (beta, "decoy-two")]),
            ),
            create_entity(&deleted, BTreeMap::new()),
            create_entity(&decoy_deleted, BTreeMap::new()),
            create_entity(&from, BTreeMap::new()),
            create_entity(&to, BTreeMap::new()),
            create_entity(&decoy_from, BTreeMap::new()),
            create_entity(&decoy_to, BTreeMap::new()),
        ],
    );
    let entities = CommittedEntities {
        update: issued_entity(&committed, &update),
        decoy_update: issued_entity(&committed, &decoy_update),
        deleted: issued_entity(&committed, &deleted),
        decoy_deleted: issued_entity(&committed, &decoy_deleted),
        from: issued_entity(&committed, &from),
        to: issued_entity(&committed, &to),
        decoy_from: issued_entity(&committed, &decoy_from),
        decoy_to: issued_entity(&committed, &decoy_to),
    };
    crate::relational_snapshot_release::release_query_snapshot(runtime, &committed.snapshot);
    entities
}

struct CommittedEntities {
    update: EntityId,
    decoy_update: EntityId,
    deleted: EntityId,
    decoy_deleted: EntityId,
    from: EntityId,
    to: EntityId,
    decoy_from: EntityId,
    decoy_to: EntityId,
}

fn issued_entity(
    committed: &worth_relational::facade::transactions::CommitResult,
    created: &CreatedEntityRef,
) -> EntityId {
    committed
        .created_entity(created)
        .expect("authoritative commit issued requested entity")
}

fn commit_relation(
    runtime: &mut RelationalRuntime,
    seed: RelationSeed<'_>,
) -> worth_relational::facade::transactions::CommitResult {
    commit(
        runtime,
        seed.batch,
        [MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId::new(31),
                client_key: ClientKey::raw(seed.key),
                source: EntityReference::Existing(seed.from),
                target: EntityReference::Existing(seed.to),
                fields: AspectFieldPatch::default(),
            },
        ))],
    )
}

struct RelationSeed<'a> {
    batch: &'a str,
    key: &'a str,
    from: EntityId,
    to: EntityId,
}

fn commit<const N: usize>(
    runtime: &mut RelationalRuntime,
    name: &str,
    intents: [MutationIntent; N],
) -> worth_relational::facade::transactions::CommitResult {
    let mut transaction = {
        let transaction_validation_input = runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .expect("main branch binding");
        runtime
            .begin_branch_transaction(
                &transaction_validation_input,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("owner-admitted transaction context")
    };
    transaction
        .push_batch(
            intents
                .into_iter()
                .fold(WorkerIntentBatch::new(name), WorkerIntentBatch::push),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("fixture truth commits")
}

struct ObservedAxes {
    update: EntityId,
    decoy_update: EntityId,
    deleted: EntityId,
    decoy_deleted: EntityId,
    from: EntityId,
    to: EntityId,
    decoy_from: EntityId,
    decoy_to: EntityId,
    relation: RelationId,
    decoy_relation: RelationId,
}

struct ObservedLocators<'a> {
    alpha: &'a AspectFieldLocator,
    beta: &'a AspectFieldLocator,
}

fn observed_facts(
    runtime: &RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    axes: ObservedAxes,
    locators: ObservedLocators<'_>,
) -> Vec<WorthQueryApplicationObservedFact> {
    let facts = vec![
        field_fact(runtime, snapshot, axes.decoy_update, locators.alpha),
        field_fact(runtime, snapshot, axes.decoy_update, locators.beta),
        field_fact(runtime, snapshot, axes.update, locators.alpha),
        field_fact(runtime, snapshot, axes.update, locators.beta),
        WorthQueryApplicationObservedFact::Entity {
            entity_id: axes.decoy_deleted,
            kind: KindId::new(12),
        },
        WorthQueryApplicationObservedFact::Entity {
            entity_id: axes.deleted,
            kind: KindId::new(12),
        },
        relation_fact(
            runtime,
            snapshot,
            RelationObservation {
                from: axes.decoy_from,
                to: axes.decoy_to,
                expected: axes.decoy_relation,
            },
        ),
        relation_fact(
            runtime,
            snapshot,
            RelationObservation {
                from: axes.from,
                to: axes.to,
                expected: axes.relation,
            },
        ),
    ];
    assert!(facts
        .iter()
        .all(|fact| fact.remains_equal_in(runtime, snapshot)));
    facts
}

fn relation_fact(
    runtime: &RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    observation: RelationObservation,
) -> WorthQueryApplicationObservedFact {
    let matching_relations = exact_relations(
        runtime,
        snapshot,
        KindId::new(31),
        observation.from,
        observation.to,
    )
    .expect("committed relation observation succeeds");
    assert_eq!(matching_relations, vec![observation.expected]);
    WorthQueryApplicationObservedFact::Relation {
        relation_kind: KindId::new(31),
        from: observation.from,
        to: observation.to,
        matching_relations,
    }
}

struct RelationObservation {
    from: EntityId,
    to: EntityId,
    expected: RelationId,
}

fn field_fact(
    runtime: &RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    locator: &AspectFieldLocator,
) -> WorthQueryApplicationObservedFact {
    WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind: KindId::new(11),
        locator: locator.clone(),
        value: observe_field_value(runtime, snapshot, entity, KindId::new(11), locator)
            .expect("committed field observation succeeds"),
    }
}

fn created(kind: u32, key: &str) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId::new(kind),
        client_key: ClientKey::raw(key),
    }
}

fn create_entity(
    created: &CreatedEntityRef,
    fields: BTreeMap<AspectFieldLocator, AspectValue>,
) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: created.partition_id,
        kind_id: created.kind_id,
        client_key: created.client_key.clone(),
        fields: AspectFieldPatch::from(fields),
    }))
}
