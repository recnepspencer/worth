use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use crate::identity::data::PartitionId;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::schema::data::EntityKindRegistration;
use crate::schema::data::KindAspectContractDeclarations;
use crate::schema::data::RelationIntegrityDeclarations;
use crate::schema::data::RelationKindRegistration;
use crate::schema::data::RelationalSchemaRegistry;
use crate::schema::data::SchemaId;
use crate::schema::data::SchemaVersionId;
use crate::symbols::data::ClientKey;
use crate::transactions::data::intents::mutation_intent::CreateIntent;
use crate::transactions::data::intents::mutation_intent::DeleteEntityIntent;
use crate::transactions::data::intents::mutation_intent::EntityMutationIntent;
use crate::transactions::data::intents::mutation_intent::MutationIntent;
use crate::transactions::data::intents::mutation_intent::RelationMutationIntent;
use crate::transactions::data::intents::mutation_intent::ReplaceEntityIntent;
use crate::transactions::data::intents::mutation_intent::UpdateRelationEndpointsIntent;
use crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use crate::transactions::data::primitives::EntityReference;
use crate::transactions::data::primitives::EntitySpec;
use crate::transactions::data::primitives::RelationSpec;
use crate::transactions::data::primitives::TransactionId;
use crate::transactions::data::primitives::WorkerIntentBatch;
use crate::validation::engine::{
    InvariantExecutionRequest, InvariantObservation, InvariantRequestProfile,
};

use super::CandidateTouchedKinds;

#[test]
fn replacement_defers_old_kind_and_adjacency_to_the_ordinary_planner() {
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime();
    let entity = create_entity(execution, &runtime, KindId(2), "old");
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
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime();
    let old_source = create_entity(execution, &runtime, KindId(2), "old-source");
    let target = create_entity(execution, &runtime, KindId(1), "target");
    let new_source = create_entity(execution, &runtime, KindId(3), "new-source");
    let relation = create_relation(execution, &runtime, old_source, target);
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
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime();
    let source = create_entity(execution, &runtime, KindId(1), "source");
    let target = create_entity(execution, &runtime, KindId(1), "target");
    create_relation(execution, &runtime, source, target);
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
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime();
    let source = create_entity(execution, &runtime, KindId(2), "source");
    let target = create_entity(execution, &runtime, KindId(1), "target");
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
    let mut access = crate::validation::data::descriptor::CustomInvariantAccessContract {
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
    assert!(kinds.may_affect(
        &crate::validation::data::descriptor::CustomInvariantAccessContract {
            read_entity_kinds: vec![KindId(2)],
            read_relation_kinds: Vec::new(),
            affected_entity_kinds: vec![KindId(2)],
            affected_relation_kinds: Vec::new(),
            include_relation_endpoint_entity_touches: false,
        }
    ));
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
    CandidateTouchedKinds::from_request(&request, None)
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
        .expect("valid native runtime configuration")
}

fn create_entity(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    kind: KindId,
    key: &str,
) -> EntityId {
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
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("entity stages");
    transaction
        .commit(execution, runtime)
        .expect("entity publishes")
        .changed_records
        .iter()
        .find_map(|record| match record {
            crate::transactions::data::primitives::RecordRef::Entity(id) => Some(*id),
            _ => None,
        })
        .expect("entity identity")
}

fn create_relation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: EntityId,
    target: EntityId,
) -> RelationId {
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("edge").push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(4),
                    client_key: ClientKey::raw("edge"),
                    source: EntityReference::Existing(source),
                    target: EntityReference::Existing(target),
                    fields: Default::default(),
                },
            ))),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("relation stages");
    transaction
        .commit(execution, runtime)
        .expect("relation publishes")
        .changed_records
        .iter()
        .find_map(|record| match record {
            crate::transactions::data::primitives::RecordRef::Relation(id) => Some(*id),
            _ => None,
        })
        .expect("relation identity")
}
