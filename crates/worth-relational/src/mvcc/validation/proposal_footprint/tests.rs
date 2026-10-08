use worth_foundational::facade::{
    AspectValue, ContractValidationInput, PortableAspectContractBasis,
    PortableAspectPatchOperation, PortableRecordAspectPatch,
};

use crate::capabilities::AspectPlanSource;
use crate::identity::data::{KindId, PartitionId};
use crate::mvcc::ValidatedRelationalProposal;
use crate::tests::support::{
    aspect_key, create_entity, create_relation, field_key, name_field_patch,
    runtime_with_test_schema,
};
use crate::transactions::data::{
    ApplyEntityAspectPatchIntent, CreateIntent, DeleteEntityIntent, EntityMutationIntent,
    EntityReference, EntitySpec, MutationIntent, RecordRef, RelationMutationIntent,
    ReplaceEntityIntent, UpdateEntityFieldsIntent, UpdateRelationEndpointsIntent,
    WorkerIntentBatch,
};

use super::ValidatedMutationFootprint;

mod variant_matrix;

#[test]
fn exact_field_footprint_keeps_record_aspect_and_field_distinct() {
    let runtime = runtime_with_test_schema();
    let changed = create_entity(&runtime, "changed");
    let other = create_entity(&runtime, "other");
    let footprint = project(validate(&runtime, [update_name(changed, "after")]));

    assert!(footprint.mutates_field(&RecordRef::Entity(changed), &locator("name", "name")));
    assert!(!footprint.mutates_field(&RecordRef::Entity(changed), &locator("name", "other")));
    assert!(!footprint.mutates_field(&RecordRef::Entity(changed), &locator("other", "name")));
    assert!(!footprint.mutates_field(&RecordRef::Entity(other), &locator("name", "name")));
    assert_work(&footprint, 1, 1);
}

#[test]
fn whole_aspect_footprint_keeps_its_record_and_aspect_boundaries() {
    let runtime = runtime_with_test_schema();
    let changed = create_entity(&runtime, "changed");
    let other = create_entity(&runtime, "other");
    let contract = runtime
        .entity_aspect_plan(KindId(1))
        .unwrap()
        .contract_for(&aspect_key("name"))
        .unwrap()
        .clone();
    let patch = PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(&contract),
        value: ContractValidationInput::Scalar(AspectValue::String("after".into())),
    }]);
    let footprint = project(validate(
        &runtime,
        [MutationIntent::Entity(
            EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                entity_id: changed,
                aspect_patch: patch,
            }),
        )],
    ));

    assert!(footprint.mutates_field(&RecordRef::Entity(changed), &locator("name", "any")));
    assert!(!footprint.mutates_field(&RecordRef::Entity(changed), &locator("other", "any")));
    assert!(!footprint.mutates_field(&RecordRef::Entity(other), &locator("name", "any")));
    assert_work(&footprint, 1, 1);
}

#[test]
fn replace_and_delete_footprints_cover_only_the_exact_whole_record() {
    let runtime = runtime_with_test_schema();
    let changed = create_entity(&runtime, "changed");
    let other = create_entity(&runtime, "other");
    let replace = MutationIntent::Entity(EntityMutationIntent::Replace(ReplaceEntityIntent {
        entity_id: changed,
        replacement: entity_spec("replacement"),
    }));
    let replaced = project(validate(&runtime, [replace]));
    assert_whole_record(&replaced, changed, other);

    let deleted = project(validate(
        &runtime,
        [MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id: changed },
        ))],
    ));
    assert_whole_record(&deleted, changed, other);
}

#[test]
fn create_only_and_endpoint_only_intents_examine_work_but_name_no_prior_field() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let old_target = create_entity(&runtime, "old-target");
    let new_target = create_entity(&runtime, "new-target");
    let relation = create_relation(&runtime, source, old_target, "edge");

    let created = project(validate(
        &runtime,
        [MutationIntent::Create(CreateIntent::Entity(entity_spec(
            "new",
        )))],
    ));
    assert!(created.is_empty());
    assert_work(&created, 1, 0);

    let endpoints = project(validate(
        &runtime,
        [MutationIntent::Relation(
            RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                relation_id: relation,
                kind_id: KindId(2),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(new_target),
            }),
        )],
    ));
    assert!(endpoints.is_empty());
    assert_work(&endpoints, 1, 0);
}

#[test]
fn mixed_breadth_reports_every_validated_intent_and_materialized_target() {
    let runtime = runtime_with_test_schema();
    let first = create_entity(&runtime, "first");
    let second = create_entity(&runtime, "second");
    let target = create_entity(&runtime, "target");
    let relation = create_relation(&runtime, first, second, "edge");
    let footprint = project(validate(
        &runtime,
        [
            update_name(first, "first-after"),
            update_name(second, "second-after"),
            MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
                UpdateRelationEndpointsIntent {
                    relation_id: relation,
                    kind_id: KindId(2),
                    source: EntityReference::Existing(first),
                    target: EntityReference::Existing(target),
                },
            )),
        ],
    ));

    assert!(footprint.mutates_field(&RecordRef::Entity(first), &locator("name", "name")));
    assert!(footprint.mutates_field(&RecordRef::Entity(second), &locator("name", "name")));
    assert_work(&footprint, 3, 2);
}

#[test]
fn omitted_demand_performs_no_footprint_scan_or_materialization() {
    let runtime = runtime_with_test_schema();
    let first = create_entity(&runtime, "first");
    let second = create_entity(&runtime, "second");
    let validated = validate(
        &runtime,
        [
            update_name(first, "first-after"),
            update_name(second, "second-after"),
        ],
    );

    let not_requested = validated.mutation_footprint::<()>(None);

    assert!(matches!(
        not_requested,
        super::ValidatedMutationFootprintProjection::NotRequested(_)
    ));
    assert!(not_requested.projected().is_none());
    assert_eq!(not_requested.work().validated_intents_examined(), 0);
    assert_eq!(not_requested.work().mutation_targets_materialized(), 0);
}

fn validate(
    runtime: &crate::runtime::RelationalRuntime,
    intents: impl IntoIterator<Item = MutationIntent>,
) -> ValidatedRelationalProposal {
    let batch = intents.into_iter().fold(
        WorkerIntentBatch::new("footprint-owner-proof"),
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
        .validate(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("owner validates fixture mutation")
}

fn project(validated: ValidatedRelationalProposal) -> ValidatedMutationFootprint {
    validated
        .mutation_footprint(Some(&()))
        .into_projected()
        .expect("explicit demand projects footprint")
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

fn locator(aspect: &str, field: &str) -> worth_foundational::facade::AspectFieldLocator {
    crate::transactions::data::planned_single_field_locator(aspect_key(aspect), field_key(field))
}

fn assert_whole_record(
    footprint: &ValidatedMutationFootprint,
    changed: crate::identity::data::EntityId,
    other: crate::identity::data::EntityId,
) {
    assert!(footprint.mutates_field(&RecordRef::Entity(changed), &locator("any", "field")));
    assert!(!footprint.mutates_field(&RecordRef::Entity(other), &locator("any", "field")));
    assert_work(footprint, 1, 1);
}

fn assert_work(footprint: &ValidatedMutationFootprint, intents: usize, targets: usize) {
    assert_eq!(footprint.work().validated_intents_examined(), intents);
    assert_eq!(footprint.work().mutation_targets_materialized(), targets);
}
