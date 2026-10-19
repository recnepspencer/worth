use crate::identity::data::{KindId, PartitionId};
use crate::tests::support::{
    create_entity, create_relation, name_field_patch, runtime_with_test_schema,
};
use crate::transactions::data::{
    CreateIntent, DeleteEntityIntent, EntityMutationIntent, EntityReference, EntitySpec,
    MutationIntent, RelationMutationIntent, UpdateEntityFieldsIntent,
    UpdateRelationEndpointsIntent, WorkerIntentBatch,
};

use super::ValidatedMutationTouch;

#[test]
fn create_and_update_project_exact_entity_touch_loci() {
    let host_request = crate::test_execution_policy::serial_request();
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
    let host_request = crate::test_execution_policy::serial_request();
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
    runtime: &crate::runtime::RelationalRuntime,
    intents: impl IntoIterator<Item = MutationIntent>,
) -> crate::mvcc::ValidatedRelationalProposal {
    let batch = intents.into_iter().fold(
        WorkerIntentBatch::new("validated-touch-owner-proof"),
        WorkerIntentBatch::push,
    );
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
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

fn update_name(entity_id: crate::identity::data::EntityId, value: &str) -> MutationIntent {
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
        client_key: crate::symbols::data::ClientKey::raw(key),
        fields: name_field_patch(key),
    }
}
