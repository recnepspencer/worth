use super::proposal_records::{create_relation, name_field_patch};
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_testkit_relational::{
    aspect_storage::publications::begin_main_transaction,
    durable_publication::memory_runtime as runtime_with_test_schema,
    ordinary_records::create_entity,
    shared_relational_certification_test_execution_policy::serial_request,
};

use worth_relational::facade::transactions::{
    CreateIntent, DeleteEntityIntent, EntityMutationIntent, EntityReference, EntitySpec,
    MutationIntent, RelationMutationIntent, UpdateEntityFieldsIntent,
    UpdateRelationEndpointsIntent, WorkerIntentBatch,
};

use worth_relational::facade::mvcc::ValidatedMutationTouch;

mod initialization;

#[test]
fn create_and_update_project_exact_entity_touch_loci() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let existing = create_entity(execution, &runtime, "existing");
    let validated = validate(
        execution,
        &runtime,
        [
            MutationIntent::Create(CreateIntent::Entity(entity_spec("created"))),
            update_name(existing, "updated"),
        ],
    );
    let projected = validated
        .mutation_touches(execution)
        .expect("exact native touch projection");

    assert!(projected
        .touches()
        .contains(&ValidatedMutationTouch::CreateEntity { kind: KindId(1) }));
    assert_eq!(
        projected
            .touches()
            .iter()
            .filter(|touch| matches!(touch, ValidatedMutationTouch::WriteEntityField { .. }))
            .count(),
        1,
        "create and update of the same semantic field deduplicate to one touch locus"
    );
    assert_eq!(projected.work().validated_intents_examined(), 2);
    assert_eq!(projected.work().mutation_targets_materialized(), 2);
    assert_eq!(projected.work().owner_state_lookups(), 1);
}

#[test]
fn delete_and_endpoint_rewrite_preserve_unlink_and_link_meaning() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let source = create_entity(execution, &runtime, "source");
    let old_target = create_entity(execution, &runtime, "old-target");
    let new_target = create_entity(execution, &runtime, "new-target");
    let relation = create_relation(execution, &runtime, source, old_target, "edge");
    let validated = validate(
        execution,
        &runtime,
        [
            MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
                entity_id: old_target,
            })),
            MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
                UpdateRelationEndpointsIntent {
                    relation_id: relation,
                    kind_id: KindId(2),
                    source: EntityReference::Existing(source),
                    target: EntityReference::Existing(new_target),
                },
            )),
        ],
    );
    let projected = validated
        .mutation_touches(execution)
        .expect("exact native touch projection");

    assert!(projected
        .touches()
        .contains(&ValidatedMutationTouch::DeleteEntity { kind: KindId(1) }));
    assert!(projected
        .touches()
        .contains(&ValidatedMutationTouch::UnlinkRelation { kind: KindId(2) }));
    assert!(projected
        .touches()
        .contains(&ValidatedMutationTouch::LinkRelation { kind: KindId(2) }));
    assert_eq!(projected.work().validated_intents_examined(), 2);
    assert_eq!(projected.work().mutation_targets_materialized(), 3);
}

fn validate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    intents: impl IntoIterator<Item = MutationIntent>,
) -> worth_relational::facade::mvcc::ValidatedRelationalProposal {
    let batch = intents.into_iter().fold(
        WorkerIntentBatch::new("validated-touch-owner-proof"),
        WorkerIntentBatch::push,
    );
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .validate(execution, runtime)
        .expect("owner validates touch projection fixture")
}

fn update_name(
    entity_id: worth_relational::facade::identity::EntityId,
    value: &str,
) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::UpdateFields(
        UpdateEntityFieldsIntent {
            entity_id,
            fields: name_field_patch(value),
        },
    ))
}

fn entity_spec(key: &str) -> EntitySpec {
    EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: worth_relational::facade::symbols::ClientKey::raw(key),
        fields: name_field_patch(key),
    }
}
