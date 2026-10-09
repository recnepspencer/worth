use worth_foundational::facade::{
    AspectValue, ContractValidationInput, PortableAspectContractBasis, PortableAspectFieldSet,
    PortableAspectPatchOperation, PortableRecordAspectPatch, StructAspectValue,
};

use crate::capabilities::AspectPlanSource;
use crate::identity::data::KindId;
use crate::tests::support::{
    aspect_key, create_entity, create_relation, entity_summary_struct_aspect, field_key,
    runtime_with_test_schema, AspectSchemaFixture,
};
use crate::transactions::data::{
    ApplyEntityAspectPatchIntent, ApplyRelationAspectPatchIntent, DeleteRelationIntent,
    EntityMutationIntent, MutationIntent, RecordRef, RelationMutationIntent, WorkerIntentBatch,
};

use super::{assert_work, locator, project, validate};

#[test]
fn entity_field_patch_materializes_only_selected_fields_on_its_exact_record_type() {
    let runtime = runtime_with_struct_aspects();
    let changed = create_entity(&runtime, "changed");
    let other = create_entity(&runtime, "other");
    let relation = create_relation(&runtime, changed, other, "edge");
    let contract = summary_contract(&runtime);
    commit_intent(
        &runtime,
        entity_summary_patch(changed, whole_summary_patch(&contract, "before", "open")),
    );
    let footprint = project(validate(
        &runtime,
        [entity_summary_patch(
            changed,
            selected_summary_patch(&contract, &[("title", "after"), ("status", "closed")]),
        )],
    ));

    assert!(footprint.mutates_field(&RecordRef::Entity(changed), &locator("summary", "title")));
    assert!(footprint.mutates_field(&RecordRef::Entity(changed), &locator("summary", "status")));
    assert!(!footprint.mutates_field(&RecordRef::Entity(changed), &locator("summary", "other")));
    assert!(!footprint.mutates_field(&RecordRef::Entity(other), &locator("summary", "title")));
    assert!(!footprint.mutates_field(&RecordRef::Relation(relation), &locator("summary", "title")));
    assert_work(&footprint, 1, 2);
}

#[test]
fn relation_field_patch_materializes_only_selected_fields_on_its_exact_record_type() {
    let runtime = runtime_with_struct_aspects();
    let source = create_entity(&runtime, "source");
    let first_target = create_entity(&runtime, "first-target");
    let second_target = create_entity(&runtime, "second-target");
    let changed = create_relation(&runtime, source, first_target, "changed");
    let other = create_relation(&runtime, source, second_target, "other");
    let contract = summary_contract(&runtime);
    commit_intent(
        &runtime,
        relation_summary_patch(changed, whole_summary_patch(&contract, "before", "open")),
    );
    let footprint = project(validate(
        &runtime,
        [relation_summary_patch(
            changed,
            selected_summary_patch(&contract, &[("status", "closed")]),
        )],
    ));

    assert!(footprint.mutates_field(&RecordRef::Relation(changed), &locator("summary", "status")));
    assert!(!footprint.mutates_field(&RecordRef::Relation(changed), &locator("summary", "title")));
    assert!(!footprint.mutates_field(&RecordRef::Relation(other), &locator("summary", "status")));
    assert!(!footprint.mutates_field(&RecordRef::Entity(source), &locator("summary", "status")));
    assert_work(&footprint, 1, 1);
}

#[test]
fn relation_whole_aspect_footprint_keeps_record_type_and_aspect_distinct() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let first_target = create_entity(&runtime, "first-target");
    let second_target = create_entity(&runtime, "second-target");
    let changed = create_relation(&runtime, source, first_target, "changed");
    let other = create_relation(&runtime, source, second_target, "other");
    let contract = runtime
        .relation_aspect_plan(KindId(2))
        .unwrap()
        .contract_for(&aspect_key("label"))
        .unwrap();
    let patch = PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(&contract),
        value: ContractValidationInput::Scalar(AspectValue::String("after".into())),
    }]);
    let footprint = project(validate(&runtime, [relation_summary_patch(changed, patch)]));

    assert!(footprint.mutates_field(&RecordRef::Relation(changed), &locator("label", "any")));
    assert!(!footprint.mutates_field(&RecordRef::Relation(changed), &locator("other", "any")));
    assert!(!footprint.mutates_field(&RecordRef::Relation(other), &locator("label", "any")));
    assert!(!footprint.mutates_field(&RecordRef::Entity(source), &locator("label", "any")));
    assert_work(&footprint, 1, 1);
}

#[test]
fn relation_delete_footprint_covers_only_the_exact_relation_record() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let first_target = create_entity(&runtime, "first-target");
    let second_target = create_entity(&runtime, "second-target");
    let changed = create_relation(&runtime, source, first_target, "changed");
    let other = create_relation(&runtime, source, second_target, "other");
    let footprint = project(validate(
        &runtime,
        [MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent {
                relation_id: changed,
            },
        ))],
    ));

    assert!(footprint.mutates_field(&RecordRef::Relation(changed), &locator("any", "field")));
    assert!(!footprint.mutates_field(&RecordRef::Relation(other), &locator("any", "field")));
    assert!(!footprint.mutates_field(&RecordRef::Entity(source), &locator("any", "field")));
    assert_work(&footprint, 1, 1);
}

fn runtime_with_struct_aspects() -> crate::runtime::RelationalRuntime {
    let entity_binding = entity_summary_struct_aspect(aspect_key("summary"), field_key("summary"));
    let relation_binding = crate::schema::data::DeclaredAspectContractBinding {
        binding: worth_foundational::facade::AspectBinding::RelationField {
            field: field_key("summary"),
        },
        contract: entity_binding.contract.clone(),
    };
    AspectSchemaFixture {
        entity_aspects: vec![entity_binding],
        relation_aspects: vec![relation_binding],
        ..AspectSchemaFixture::default()
    }
    .build_runtime()
}

fn summary_contract(
    runtime: &crate::runtime::RelationalRuntime,
) -> worth_foundational::facade::AspectContract {
    runtime
        .entity_aspect_plan(KindId(1))
        .unwrap()
        .contract_for(&aspect_key("summary"))
        .unwrap()
        .clone()
}

fn whole_summary_patch(
    contract: &worth_foundational::facade::AspectContract,
    title: &str,
    status: &str,
) -> PortableRecordAspectPatch {
    let value = StructAspectValue::new([
        (field_key("title"), AspectValue::String(title.into())),
        (field_key("status"), AspectValue::String(status.into())),
    ])
    .unwrap();
    PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(contract),
        value: ContractValidationInput::Struct(value),
    }])
}

fn selected_summary_patch(
    contract: &worth_foundational::facade::AspectContract,
    fields: &[(&str, &str)],
) -> PortableRecordAspectPatch {
    let selected_fields = fields.iter().map(|(field, _)| field_key(field)).collect();
    let field_sets = fields
        .iter()
        .map(|(field, value)| {
            PortableAspectFieldSet::new(field_key(field), AspectValue::String((*value).into()))
        })
        .collect();
    PortableRecordAspectPatch::new([PortableAspectPatchOperation::PatchFields {
        basis: PortableAspectContractBasis::from_contract(contract),
        selected_fields,
        field_sets,
        field_clears: Vec::new(),
    }])
}

fn entity_summary_patch(
    entity_id: crate::identity::data::EntityId,
    aspect_patch: PortableRecordAspectPatch,
) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(
        ApplyEntityAspectPatchIntent {
            entity_id,
            aspect_patch,
        },
    ))
}

fn relation_summary_patch(
    relation_id: crate::identity::data::RelationId,
    aspect_patch: PortableRecordAspectPatch,
) -> MutationIntent {
    MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(
        ApplyRelationAspectPatchIntent {
            relation_id,
            aspect_patch,
        },
    ))
}

fn commit_intent(runtime: &crate::runtime::RelationalRuntime, intent: MutationIntent) {
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("footprint-setup").push(intent),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("fixture state commits");
}
