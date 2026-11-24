//! Refusal summaries derive from genuine native artifacts, never payload Debug.
use super::*;

#[test]
fn present_cell_refusals_identify_duplicate_binding_and_operation_form() {
    let request = crate::execution::test_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let fixture = scalar_birth_fixture(execution, "payload-and-client-key-sentinel");
    let (id, original, plan, payload) = (
        fixture.id,
        &fixture.envelope,
        &fixture.plan,
        fixture.payload.as_ref().unwrap(),
    );
    assert!(validate(execution, original, id, plan, Some(payload)).is_ok());
    let operation = original.patch.authoritative_record_patches[0]
        .authoritative_patch
        .full_grammar_operations()
        .iter()
        .find(|op| op.aspect_key() == &aspect_key("name"))
        .unwrap()
        .clone();
    let mut duplicate = original.independent_fixture_image();
    let mut operations = duplicate.patch.authoritative_record_patches[0]
        .authoritative_patch
        .full_grammar_operations()
        .to_vec();
    operations.push(operation.clone());
    duplicate
        .body_mut_for_test()
        .patch
        .authoritative_record_patches[0]
        .authoritative_patch = PublishedAuthoritativePatch::new(operations);
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

    let mut wrong_binding = original.independent_fixture_image();
    let mut op = operation.clone();
    let Operation::WholeAspectSet { binding, .. } = &mut op else {
        panic!("actual scalar set")
    };
    *binding = AspectBinding::StructuralFacet;
    wrong_binding
        .body_mut_for_test()
        .patch
        .authoritative_record_patches[0]
        .authoritative_patch = PublishedAuthoritativePatch::new(vec![op]);
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
    let mut cleared = original.independent_fixture_image();
    cleared
        .body_mut_for_test()
        .patch
        .authoritative_record_patches[0]
        .authoritative_patch =
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
}

#[test]
fn birth_field_clear_refuses_with_the_selected_struct_operation_summary() {
    let request = crate::execution::test_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    use worth_foundational::facade::{StructAspectValue, StructFieldBuilder};

    let fields = StructFieldBuilder::default()
        .required("title", ScalarAspectType::String)
        .optional("status", ScalarAspectType::String)
        .finish()
        .unwrap();
    let contract = worth_foundational::facade::aspects()
        .contract()
        .for_key(aspect_key("summary"))
        .identified_by(AspectIdentity(2))
        .at_revision(AspectContractRevision(1))
        .struct_aspect(fields);
    let value = StructAspectValue::new([
        (
            field_key("title"),
            AspectValue::String("private-value-sentinel".into()),
        ),
        (
            field_key("status"),
            AspectValue::String("private-value-sentinel".into()),
        ),
    ])
    .unwrap();
    let fixture = birth_fixture(
        execution,
        "private-key-sentinel",
        Some((contract, ContractValidationInput::Struct(value))),
    );
    let (id, mut envelope, plan, payload) = (
        fixture.id,
        fixture.envelope,
        &fixture.plan,
        fixture.payload.as_ref().unwrap(),
    );
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
    envelope
        .body_mut_for_test()
        .patch
        .authoritative_record_patches[0]
        .authoritative_patch = PublishedAuthoritativePatch::new(vec![Operation::FieldLevelPatch {
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
}
