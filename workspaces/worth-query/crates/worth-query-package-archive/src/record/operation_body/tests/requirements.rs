//! V7 retains both expressions and exact typed collection/field correspondence.
use super::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationFieldPresence, ApplicationOperationBodyRequirementDirection as Direction,
    ApplicationOperationBodyRequirementExpression as Expression,
    ApplicationOperationBodyRequirementOperand as Operand,
    ApplicationOperationBodyRequirementProgram as Requirement,
    ApplicationOperationBodyRequirementSource as Source,
};

#[test]
fn conditional_requirement_v7_roundtrips_and_old_record_leaves_rejected_trailing_bytes() {
    let reader =
        worth_foundational::expression_api::expressions().reading_within(Expression::PROFILE);
    let requirement = Requirement::from_untrusted(
        "members-minimum".to_owned(),
        vec![
            Operand::from_untrusted(
                "active".to_owned(),
                Source::Field {
                    field: ApplicationFieldBindingLocus::new("Item", "Facts", "Active"),
                    binding: WorthQueryPortableTypeIdentity::from_untrusted(
                        "worth.rust.bool".to_owned(),
                    ),
                    scalar_family: ScalarAspectType::Bool,
                    presence: ApplicationFieldPresence::Required,
                    presence_only: false,
                },
            ),
            Operand::from_untrusted(
                "members".to_owned(),
                Source::RelatedItemCount {
                    relation: "Members".to_owned(),
                    from: "Item".to_owned(),
                    to: "Member".to_owned(),
                    direction: Direction::Outgoing,
                },
            ),
        ],
        Some(reader.parse("active").unwrap().encode()),
        reader
            .parse("members >= exact_cast<UInt64>(1)")
            .unwrap()
            .encode(),
        "members".to_owned(),
    )
    .unwrap();
    let body = ApplicationOperationBodyProgram::from_untrusted_with_requirements(
        vec![ApplicationOperationBodyScopeField::Assign {
            field: ApplicationFieldBindingLocus::new("Item", "Facts", "Ready"),
            binding: WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.bool".to_owned()),
            value: AspectValue::Bool(true),
        }],
        None,
        vec![requirement],
    );
    let mut output = BinaryOutput::with_capacity(1024);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &7_u16.to_be_bytes());
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
    bytes[..2].copy_from_slice(&6_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let mut input = BinaryInput::new(&bytes);
    assert!(decode(&mut input, &mut budget)
        .unwrap()
        .requirements()
        .is_empty());
    // Record admission requires exact consumption; old carriage cannot silently
    // accept the appended requirement group as a v6 body.
    assert!(!input.is_finished());
}
