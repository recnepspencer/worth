//! Refusal summaries derive from genuine native artifacts, never payload Debug.
use super::*;
use crate::publication::patch::data::PublishedAuthoritativePatch;

#[test]
fn present_cell_refusals_identify_duplicate_binding_and_operation_form() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![entity_field_aspect(aspect_key("name"), field_key("name"))],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let result = create_entity_outcome(execution, &runtime, "payload-and-client-key-sentinel");
    let id = changed_entities(&result)[0];
    let original = &result.publication().envelope;
    let basis = test_owner_main_basis(&runtime).unwrap();
    let plan = basis
        .inner
        .root
        .schema_authority()
        .entity_aspect_plan(KindId(1))
        .unwrap();
    let partition = basis.inner.root.partition_state(id.partition_id).unwrap();
    let row = partition.entity_arena.get(&id).unwrap();
    let payload = row.extra().authoritative_aspect_state.as_ref().unwrap();
    assert!(validate(execution, original, id, plan, Some(payload)).is_ok());
    let operation = original.patch.authoritative_record_patches[0]
        .authoritative_patch
        .full_grammar_operations()
        .iter()
        .find(|op| op.aspect_key() == &aspect_key("name"))
        .unwrap()
        .clone();
    let mut duplicate = original.as_ref().clone();
    let mut operations = duplicate.patch.authoritative_record_patches[0]
        .authoritative_patch
        .full_grammar_operations()
        .to_vec();
    operations.push(operation.clone());
    duplicate.patch.authoritative_record_patches[0].authoritative_patch =
        PublishedAuthoritativePatch::new(operations);
    let failure = validate(execution, &duplicate, id, plan, Some(payload)).unwrap_err();
    assert!(matches!(
        failure,
        OutputPayloadCoverageFailure::Unsupported(UnsupportedPayload::DuplicateOperations(_))
    ));
    let error =
        super::super::super::readmission::coverage_commit_stop(failure, &duplicate, id, plan, 3);
    let detail = error.detail();
    assert!(detail.contains("reason=duplicate-present-aspect-operations"));
    assert!(detail.contains("first=[operation_ordinal="));
    assert!(detail.contains("second=[operation_ordinal="));
    assert!(detail.contains("member_ordinal=3"));
    assert_eq!(error.context().affected_records, [RecordRef::Entity(id)]);
    assert!(!detail.contains("payload-and-client-key-sentinel"));
    assert!(!detail.contains("aspect_key="));

    let mut wrong_binding = original.as_ref().clone();
    let mut op = operation.clone();
    let Operation::WholeAspectSet { binding, .. } = &mut op else {
        panic!("actual scalar set")
    };
    *binding = AspectBinding::StructuralFacet;
    wrong_binding.patch.authoritative_record_patches[0].authoritative_patch =
        PublishedAuthoritativePatch::new(vec![op]);
    let failure = validate(execution, &wrong_binding, id, plan, Some(payload)).unwrap_err();
    assert!(matches!(
        failure,
        OutputPayloadCoverageFailure::Unsupported(UnsupportedPayload::TargetBinding { .. })
    ));
    let detail = super::super::super::readmission::coverage_commit_stop(
        failure,
        &wrong_binding,
        id,
        plan,
        0,
    )
    .detail();
    assert!(detail.contains("reason=unsupported-target-binding"));
    assert!(detail.contains("binding=structural-facet"));

    let Operation::WholeAspectSet {
        aspect_key,
        aspect_identity,
        contract_revision,
        binding,
        ..
    } = operation
    else {
        panic!("actual scalar set")
    };
    let mut cleared = original.as_ref().clone();
    cleared.patch.authoritative_record_patches[0].authoritative_patch =
        PublishedAuthoritativePatch::new(vec![Operation::WholeAspectClear {
            aspect_key,
            aspect_identity,
            contract_revision,
            binding,
        }]);
    let failure = validate(execution, &cleared, id, plan, Some(payload)).unwrap_err();
    assert!(matches!(
        failure,
        OutputPayloadCoverageFailure::Unsupported(UnsupportedPayload::OperationValue { .. })
    ));
    let detail =
        super::super::super::readmission::coverage_commit_stop(failure, &cleared, id, plan, 0)
            .detail();
    assert!(detail.contains("reason=unsupported-operation-value-form"));
    assert!(detail.contains("present=scalar"));
    assert!(detail.contains("operation=whole-clear"));
    release_test_commit_snapshot(&runtime, &result);
}

#[test]
fn birth_field_clear_refuses_with_the_selected_struct_operation_summary() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let fixture = AspectSchemaFixture {
        entity_aspects: vec![entity_summary_struct_aspect(
            aspect_key("summary"),
            field_key("summary"),
        )],
        ..AspectSchemaFixture::default()
    };
    let runtime = fixture.build_runtime();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            crate::transactions::data::WorkerIntentBatch::new("actual-struct-birth").push(
                crate::transactions::data::MutationIntent::Create(
                    crate::transactions::data::CreateIntent::Entity(
                        crate::transactions::data::EntitySpec {
                            partition_id: crate::identity::data::PartitionId::main(),
                            kind_id: KindId(1),
                            client_key: crate::symbols::data::ClientKey::raw(
                                "private-key-sentinel",
                            ),
                            fields: string_aspect_field_patch([
                                (
                                    aspect_key("summary"),
                                    field_key("title"),
                                    "private-value-sentinel",
                                ),
                                (
                                    aspect_key("summary"),
                                    field_key("status"),
                                    "private-value-sentinel",
                                ),
                            ]),
                        },
                    ),
                ),
            ),
            execution.allocation_policy(),
        )
        .unwrap();
    let result = transaction.commit(execution, &runtime).unwrap();
    let id = changed_entities(&result)[0];
    let basis = test_owner_main_basis(&runtime).unwrap();
    let plan = basis
        .inner
        .root
        .schema_authority()
        .entity_aspect_plan(KindId(1))
        .unwrap();
    let partition = basis.inner.root.partition_state(id.partition_id).unwrap();
    let row = partition.entity_arena.get(&id).unwrap();
    let payload = row.extra().authoritative_aspect_state.as_ref().unwrap();
    let mut envelope = result.publication().envelope.as_ref().clone();
    assert!(validate(execution, &envelope, id, plan, Some(payload)).is_ok());
    let Operation::WholeAspectSet {
        aspect_key,
        aspect_identity,
        contract_revision,
        binding,
        value: Value::Struct(value),
    } = envelope.patch.authoritative_record_patches[0]
        .authoritative_patch
        .full_grammar_operations()[0]
        .clone()
    else {
        panic!("native creation groups initial fields into one whole struct set")
    };
    let sets = value
        .fields()
        .map(
            |(field, value)| crate::publication::patch::data::PublishedAuthoritativeFieldSet {
                field: field.clone(),
                value: value.clone(),
            },
        )
        .collect();
    envelope.patch.authoritative_record_patches[0].authoritative_patch =
        PublishedAuthoritativePatch::new(vec![Operation::FieldLevelPatch {
            aspect_key,
            aspect_identity,
            contract_revision,
            binding,
            field_sets: sets,
            field_clears: vec![field_key("status")],
        }]);
    let failure = validate(execution, &envelope, id, plan, Some(payload)).unwrap_err();
    assert!(matches!(
        failure,
        OutputPayloadCoverageFailure::Unsupported(UnsupportedPayload::OperationValue { .. })
    ));
    let detail =
        super::super::super::readmission::coverage_commit_stop(failure, &envelope, id, plan, 0)
            .detail();
    assert!(detail.contains("reason=unsupported-operation-value-form"));
    assert!(detail.contains("present=struct present_fields=2"));
    assert!(detail.contains("operation=field-patch binding=entity-field sets=2 clears=1"));
    assert!(!detail.contains("private-value-sentinel"));
    assert!(!detail.contains("private-key-sentinel"));
    release_test_commit_snapshot(&runtime, &result);
}
