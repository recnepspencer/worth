//! V28 preserves one definition and two named sinks; old readers refuse it.
use super::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationFieldPresence, ApplicationOperationBodyExpression as Expression,
    ApplicationOperationBodyRequirementOperand as Operand,
    ApplicationOperationBodyRequirementSource as Source,
    ApplicationOperationBodyValueProgram as Value,
};

#[test]
fn shared_computed_value_v28_roundtrips_and_cannot_downgrade_to_v27() {
    let binding = WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.u64".into());
    let field = ApplicationFieldBindingLocus::new("Item", "Facts", "Revision");
    let value = Value::from_untrusted(
        "next".into(),
        binding.clone(),
        vec![Operand::from_untrusted(
            "revision".into(),
            Source::Field {
                field: field.clone(),
                binding: binding.clone(),
                scalar_family: ScalarAspectType::UInt64,
                presence: ApplicationFieldPresence::Required,
                presence_only: false,
            },
        )],
        worth_foundational::expression_api::expressions()
            .reading_within(Expression::PROFILE)
            .parse("revision + uint64(1)")
            .unwrap()
            .encode(),
    )
    .unwrap();
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Item".into(),
        vec![ApplicationOperationBodyScopeField::AssignValue {
            field,
            binding: binding.clone(),
            value: "next".into(),
        }],
        Some(ApplicationOperationBodyCreation::from_untrusted(
            "Audit".into(),
            "audit".into(),
            vec![ApplicationOperationBodyCreatedField::Computed {
                field: ApplicationFieldBindingLocus::new("Audit", "Facts", "Revision"),
                binding,
                value: "next".into(),
            }],
        )),
        vec![],
    )
    .with_untrusted_values(vec![value]);
    let mut output = BinaryOutput::with_capacity(4096);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &28_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let mut input = BinaryInput::new(&bytes);
    assert_eq!(decode(&mut input, &mut budget).unwrap(), body);
    assert!(input.is_finished());
    let mut measure = crate::binary_encoding::BinaryEncodingMeasure::default();
    write(&mut measure, &body).unwrap();
    assert_eq!(measure.nested_entries(), budget.finish().nested_entries());
    bytes[..2].copy_from_slice(&27_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    assert_eq!(
        decode(&mut BinaryInput::new(&bytes), &mut budget)
            .unwrap_err()
            .kind(),
        Kind::UnsupportedRecordVariant
    );
}
