use std::collections::BTreeMap;

use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::AspectValue;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::identity::RelationId;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::CreatedEntityRef;
use worth_relational::facade::transactions::EntityReference;
use worth_relational::facade::transactions::EntitySpec;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::RecordRef;
use worth_relational::facade::transactions::RelationSpec;
use worth_relational::facade::transactions::WorkerIntentBatch;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::observation::exact_relations;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::observation::observe_field_value;

pub struct CommittedObservationWorld {
    pub update_entity: EntityId,
    pub deleted_entity: EntityId,
    pub relation_from: EntityId,
    pub relation_to: EntityId,
    pub deleted_relation: RelationId,
    pub facts: Vec<WorthQueryApplicationObservedFact>,
}

pub fn compile_committed_observations(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    alpha: &AspectFieldLocator,
    beta: &AspectFieldLocator,
) -> CommittedObservationWorld {
    let mut runtime = crate::domain_computation::primary_graph::application_attempt::provider_binding::semantic_model_tests::authoritative_truth::schema::fixture_runtime();
    let entities = commit_entities(execution, &mut runtime, alpha, beta);
    let decoy_relation = commit_relation(
        execution,
        &mut runtime,
        RelationSeed {
            batch: "provider-effect-decoy-relation",
            key: "decoy-edge",
            from: entities.decoy_from,
            to: entities.decoy_to,
        },
    );
    let relation_commit = commit_relation(
        execution,
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
    let facts = observed_facts(
        execution,
        &runtime,
        &relation_commit.snapshot,
        axes,
        locators,
    );
    worth_query_execution::court_doors_for_test::relational_snapshot_release::release_query_snapshot(
        &mut runtime,
        &relation_commit.snapshot,
    );
    worth_query_execution::court_doors_for_test::relational_snapshot_release::release_query_snapshot(
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

pub fn created_relation(
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

pub fn commit_entities(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        execution,
        runtime,
        "provider-effect-observed-entities",
        [
            create_entity(
                &update,
                crate::domain_computation::primary_graph::application_attempt::provider_binding::semantic_model_tests::world::values([(alpha, "one-before"), (beta, "two-before")]),
            ),
            create_entity(
                &decoy_update,
                crate::domain_computation::primary_graph::application_attempt::provider_binding::semantic_model_tests::world::values([(alpha, "decoy-one"), (beta, "decoy-two")]),
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
    worth_query_execution::court_doors_for_test::relational_snapshot_release::release_query_snapshot(
        runtime,
        &committed.snapshot,
    );
    entities
}

pub struct CommittedEntities {
    pub update: EntityId,
    pub decoy_update: EntityId,
    pub deleted: EntityId,
    pub decoy_deleted: EntityId,
    pub from: EntityId,
    pub to: EntityId,
    pub decoy_from: EntityId,
    pub decoy_to: EntityId,
}

pub fn issued_entity(
    committed: &worth_relational::facade::transactions::CommitResult,
    created: &CreatedEntityRef,
) -> EntityId {
    committed
        .created_entity(created)
        .expect("authoritative commit issued requested entity")
}

pub fn commit_relation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut RelationalRuntime,
    seed: RelationSeed<'_>,
) -> worth_relational::facade::transactions::CommitResult {
    commit(
        execution,
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

pub struct RelationSeed<'a> {
    pub batch: &'a str,
    pub key: &'a str,
    pub from: EntityId,
    pub to: EntityId,
}

pub fn commit<const N: usize>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        .commit(execution, runtime)
        .expect("fixture truth commits")
}

pub struct ObservedAxes {
    pub update: EntityId,
    pub decoy_update: EntityId,
    pub deleted: EntityId,
    pub decoy_deleted: EntityId,
    pub from: EntityId,
    pub to: EntityId,
    pub decoy_from: EntityId,
    pub decoy_to: EntityId,
    pub relation: RelationId,
    pub decoy_relation: RelationId,
}

pub struct ObservedLocators<'a> {
    pub alpha: &'a AspectFieldLocator,
    pub beta: &'a AspectFieldLocator,
}

pub fn observed_facts(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    axes: ObservedAxes,
    locators: ObservedLocators<'_>,
) -> Vec<WorthQueryApplicationObservedFact> {
    let facts = vec![
        field_fact(
            execution,
            runtime,
            snapshot,
            axes.decoy_update,
            locators.alpha,
        ),
        field_fact(
            execution,
            runtime,
            snapshot,
            axes.decoy_update,
            locators.beta,
        ),
        field_fact(execution, runtime, snapshot, axes.update, locators.alpha),
        field_fact(execution, runtime, snapshot, axes.update, locators.beta),
        WorthQueryApplicationObservedFact::Entity {
            entity_id: axes.decoy_deleted,
            kind: KindId::new(12),
        },
        WorthQueryApplicationObservedFact::Entity {
            entity_id: axes.deleted,
            kind: KindId::new(12),
        },
        relation_fact(
            execution,
            runtime,
            snapshot,
            RelationObservation {
                from: axes.decoy_from,
                to: axes.decoy_to,
                expected: axes.decoy_relation,
            },
        ),
        relation_fact(
            execution,
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
        .all(|fact| fact.remains_equal_in(runtime, execution, snapshot).unwrap()));
    facts
}

pub fn relation_fact(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    observation: RelationObservation,
) -> WorthQueryApplicationObservedFact {
    let matching_relations = exact_relations(
        runtime,
        execution,
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

pub struct RelationObservation {
    pub from: EntityId,
    pub to: EntityId,
    pub expected: RelationId,
}

pub fn field_fact(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    locator: &AspectFieldLocator,
) -> WorthQueryApplicationObservedFact {
    WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind: KindId::new(11),
        locator: locator.clone(),
        value: observe_field_value(
            runtime,
            execution,
            snapshot,
            entity,
            KindId::new(11),
            locator,
        )
        .expect("native field request remains live")
        .expect("committed field observation succeeds"),
    }
}

pub fn created(kind: u32, key: &str) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId::new(kind),
        client_key: ClientKey::raw(key),
    }
}

pub fn create_entity(
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

#[path = "schema/mod.rs"]
pub mod schema;
