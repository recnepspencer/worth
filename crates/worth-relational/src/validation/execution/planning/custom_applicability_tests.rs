use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::facade::identity::PartitionId;
use crate::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationKindRegistration,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::schema::data::RelationIntegrityDeclarations;
use crate::symbols::data::ClientKey;
use crate::transactions::data::{
    CreateIntent, DeleteEntityIntent, EntityMutationIntent, EntityReference, EntitySpec,
    MergedCommitPlan, MutationIntent, RelationMutationIntent, RelationSpec, ReplaceEntityIntent,
    TransactionId, UpdateRelationEndpointsIntent, WorkerIntentBatch,
};
use crate::validation::engine::{
    InvariantExecutionRequest, InvariantObservation, InvariantRequestProfile,
};

use super::CandidateTouchedKinds;

#[test]
fn replacement_defers_old_kind_and_adjacency_to_the_ordinary_planner() {
    let runtime = runtime();
    let entity = create_entity(&runtime, KindId(2), "old");
    let plan = plan(
        1,
        MutationIntent::Entity(EntityMutationIntent::Replace(ReplaceEntityIntent {
            entity_id: entity,
            replacement: EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(3),
                client_key: ClientKey::raw("replacement"),
                fields: Default::default(),
            },
        })),
    );
    assert!(touched(&runtime, &plan).is_none());
}

#[test]
fn endpoint_move_defers_old_source_kind_to_the_ordinary_planner() {
    let runtime = runtime();
    let old_source = create_entity(&runtime, KindId(2), "old-source");
    let target = create_entity(&runtime, KindId(1), "target");
    let new_source = create_entity(&runtime, KindId(3), "new-source");
    let relation = create_relation(&runtime, old_source, target);
    let plan = plan(
        2,
        MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
            UpdateRelationEndpointsIntent {
                relation_id: relation,
                kind_id: KindId(4),
                source: EntityReference::Existing(new_source),
                target: EntityReference::Existing(target),
            },
        )),
    );
    assert!(touched(&runtime, &plan).is_none());
}

#[test]
fn entity_delete_defers_cascade_applicability_to_the_ordinary_planner() {
    let runtime = runtime();
    let source = create_entity(&runtime, KindId(1), "source");
    let target = create_entity(&runtime, KindId(1), "target");
    create_relation(&runtime, source, target);
    let plan = plan(
        3,
        MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
            entity_id: source,
        })),
    );
    assert!(touched(&runtime, &plan).is_none());
}

#[test]
fn endpoint_touch_policy_preserves_default_and_allows_explicit_opt_out() {
    let runtime = runtime();
    let source = create_entity(&runtime, KindId(2), "source");
    let target = create_entity(&runtime, KindId(1), "target");
    let plan = plan(
        4,
        MutationIntent::Create(CreateIntent::Relation(RelationSpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(4),
            client_key: ClientKey::raw("unrelated-edge"),
            source: EntityReference::Existing(source),
            target: EntityReference::Existing(target),
            fields: Default::default(),
        })),
    );
    let kinds = touched(&runtime, &plan).expect("create has a complete footprint");
    let mut access = crate::validation::data::CustomInvariantAccessContract {
        read_entity_kinds: vec![KindId(2)],
        read_relation_kinds: Vec::new(),
        affected_entity_kinds: vec![KindId(2)],
        affected_relation_kinds: Vec::new(),
        include_relation_endpoint_entity_touches: true,
    };
    assert!(kinds.may_affect(&access));
    access.include_relation_endpoint_entity_touches = false;
    assert!(!kinds.may_affect(&access));
    access.read_relation_kinds.push(KindId(4));
    assert!(!kinds.may_affect(&access));
    access.read_relation_kinds.clear();
    access.affected_relation_kinds.push(KindId(4));
    assert!(kinds.may_affect(&access));
}

#[test]
fn direct_entity_creation_still_admits_entity_rule() {
    let runtime = runtime();
    let plan = plan(
        5,
        MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: ClientKey::raw("direct"),
            fields: Default::default(),
        })),
    );
    let kinds = touched(&runtime, &plan).expect("create has a complete footprint");
    assert!(
        kinds.may_affect(&crate::validation::data::CustomInvariantAccessContract {
            read_entity_kinds: vec![KindId(2)],
            read_relation_kinds: Vec::new(),
            affected_entity_kinds: vec![KindId(2)],
            affected_relation_kinds: Vec::new(),
            include_relation_endpoint_entity_touches: false,
        })
    );
}

fn touched(runtime: &RelationalRuntime, plan: &MergedCommitPlan) -> Option<CandidateTouchedKinds> {
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        runtime,
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        Some(plan),
        None,
    );
    CandidateTouchedKinds::from_request(&request)
}

fn plan(transaction: u64, intent: MutationIntent) -> MergedCommitPlan {
    MergedCommitPlan {
        transaction_id: TransactionId(transaction),
        merged_intents: vec![intent],
    }
}

fn runtime() -> RelationalRuntime {
    let mut schema = RelationalSchemaRegistry::new();
    for (kind_id, name) in [(1, "target"), (2, "old-source"), (3, "new-source")] {
        schema = schema
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(kind_id),
                kind_name: name.to_owned(),
                schema_id: SchemaId("applicability".to_owned()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
            })
            .expect("entity kind registers");
    }
    schema = schema
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(4),
            kind_name: "edge".to_owned(),
            schema_id: SchemaId("applicability".to_owned()),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
            relation_integrity: RelationIntegrityDeclarations::default(),
        })
        .expect("relation kind registers");
    RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build()
}

fn create_entity(runtime: &RelationalRuntime, kind: KindId, key: &str) -> EntityId {
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: kind,
                    client_key: ClientKey::raw(key),
                    fields: Default::default(),
                },
            ))),
        )
        .expect("entity stages");
    transaction
        .commit(runtime)
        .expect("entity publishes")
        .changed_records
        .iter()
        .find_map(|record| match record {
            crate::transactions::data::RecordRef::Entity(id) => Some(*id),
            _ => None,
        })
        .expect("entity identity")
}

fn create_relation(runtime: &RelationalRuntime, source: EntityId, target: EntityId) -> RelationId {
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(WorkerIntentBatch::new("edge").push(MutationIntent::Create(
            CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(4),
                client_key: ClientKey::raw("edge"),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(target),
                fields: Default::default(),
            }),
        )))
        .expect("relation stages");
    transaction
        .commit(runtime)
        .expect("relation publishes")
        .changed_records
        .iter()
        .find_map(|record| match record {
            crate::transactions::data::RecordRef::Relation(id) => Some(*id),
            _ => None,
        })
        .expect("relation identity")
}
