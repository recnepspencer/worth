use worth_foundational::facade::{
    AspectValue, ContractValidationInput, PortableAspectContractBasis, PortableAspectFieldSet,
    PortableAspectPatchOperation, PortableRecordAspectPatch, StructAspectValue,
};

use super::super::proposal_records::{create_entity_with_fields, create_relation_with_fields};
use super::{
    begin_main_transaction, create_entity, create_relation, runtime_with_test_schema,
    serial_request,
};
use worth_testkit_relational::aspect_storage::{
    aspect_key, entity_summary_struct_aspect, field_key, AspectSchemaFixture,
};

use worth_relational::facade::transactions::{
    ApplyEntityAspectPatchIntent, ApplyRelationAspectPatchIntent, DeleteRelationIntent,
    EntityMutationIntent, MutationIntent, RecordRef, RelationMutationIntent, WorkerIntentBatch,
};

use super::{assert_work, locator, project, validate};

#[test]
fn entity_field_patch_materializes_only_selected_fields_on_its_exact_record_type() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_struct_aspects();
    let changed = create_entity_with_fields(execution, &runtime, "changed", Default::default());
    let other = create_entity_with_fields(execution, &runtime, "other", Default::default());
    let relation = create_relation_with_fields(
        execution,
        &runtime,
        changed,
        other,
        "edge",
        Default::default(),
    );
    let contract = summary_contract();
    commit_intent(
        execution,
        &runtime,
        entity_summary_patch(changed, whole_summary_patch(&contract, "before", "open")),
    );
    let footprint = project(validate(
        execution,
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
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_struct_aspects();
    let source = create_entity_with_fields(execution, &runtime, "source", Default::default());
    let first_target =
        create_entity_with_fields(execution, &runtime, "first-target", Default::default());
    let second_target =
        create_entity_with_fields(execution, &runtime, "second-target", Default::default());
    let changed = create_relation_with_fields(
        execution,
        &runtime,
        source,
        first_target,
        "changed",
        Default::default(),
    );
    let other = create_relation_with_fields(
        execution,
        &runtime,
        source,
        second_target,
        "other",
        Default::default(),
    );
    let contract = summary_contract();
    commit_intent(
        execution,
        &runtime,
        relation_summary_patch(changed, whole_summary_patch(&contract, "before", "open")),
    );
    let footprint = project(validate(
        execution,
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
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let source = create_entity(execution, &runtime, "source");
    let first_target = create_entity(execution, &runtime, "first-target");
    let second_target = create_entity(execution, &runtime, "second-target");
    let changed = create_relation(execution, &runtime, source, first_target, "changed");
    let other = create_relation(execution, &runtime, source, second_target, "other");
    let contract = worth_testkit_relational::aspect_storage::definition::default_relation_aspects()
        .into_iter()
        .map(|(_, contract)| contract)
        .find(|contract| contract.key() == &aspect_key("label"))
        .unwrap();
    let patch = PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(&contract),
        value: ContractValidationInput::Scalar(AspectValue::String("after".into())),
    }]);
    let footprint = project(validate(
        execution,
        &runtime,
        [relation_summary_patch(changed, patch)],
    ));

    assert!(footprint.mutates_field(&RecordRef::Relation(changed), &locator("label", "any")));
    assert!(!footprint.mutates_field(&RecordRef::Relation(changed), &locator("other", "any")));
    assert!(!footprint.mutates_field(&RecordRef::Relation(other), &locator("label", "any")));
    assert!(!footprint.mutates_field(&RecordRef::Entity(source), &locator("label", "any")));
    assert_work(&footprint, 1, 1);
}

#[test]
fn relation_delete_footprint_covers_only_the_exact_relation_record() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let source = create_entity(execution, &runtime, "source");
    let first_target = create_entity(execution, &runtime, "first-target");
    let second_target = create_entity(execution, &runtime, "second-target");
    let changed = create_relation(execution, &runtime, source, first_target, "changed");
    let other = create_relation(execution, &runtime, source, second_target, "other");
    let footprint = project(validate(
        execution,
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

fn runtime_with_struct_aspects() -> worth_relational::facade::runtime::RelationalRuntime {
    let entity_binding = entity_summary_struct_aspect(aspect_key("summary"), field_key("summary"));
    let relation_binding = worth_relational::facade::schema::DeclaredAspectContractBinding {
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

fn summary_contract() -> worth_foundational::facade::AspectContract {
    entity_summary_struct_aspect(aspect_key("summary"), field_key("summary")).contract
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
    entity_id: worth_relational::facade::identity::EntityId,
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
    relation_id: worth_relational::facade::identity::RelationId,
    aspect_patch: PortableRecordAspectPatch,
) -> MutationIntent {
    MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(
        ApplyRelationAspectPatchIntent {
            relation_id,
            aspect_patch,
        },
    ))
}

fn commit_intent(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    intent: MutationIntent,
) {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("footprint-setup").push(intent),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .commit(execution, runtime)
        .expect("fixture state commits");
}
