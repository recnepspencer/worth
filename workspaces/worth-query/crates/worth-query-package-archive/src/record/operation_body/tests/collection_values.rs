//! New collection source tags must never acquire meaning in an older body profile.
use super::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationFieldPresence, ApplicationOperationBodyRequirementExpression as Expression,
    ApplicationOperationBodyRequirementOperand as Operand,
    ApplicationOperationBodyRequirementProgram as Requirement,
    ApplicationOperationBodyRequirementSource as Source,
};

#[test]
fn collection_field_values_v24_roundtrip_and_older_profiles_refuse_the_source() {
    let expression =
        worth_foundational::expression_api::expressions().reading_within(Expression::PROFILE);
    let source = Source::RelatedFieldValues {
        relation: "Items".into(),
        from: "Root".into(),
        to: "Item".into(),
        field: ApplicationFieldBindingLocus::new("Item", "Facts", "Description"),
        binding: WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.string".into()),
        scalar_family: ScalarAspectType::String,
        presence: ApplicationFieldPresence::Optional,
    };
    let requirement = Requirement::from_untrusted(
        "complete-members".into(),
        vec![Operand::from_untrusted("values".into(), source)],
        None,
        expression
            .parse("length(values) >= 1 && values.all(v, is_some(v))")
            .unwrap()
            .encode(),
        "values".into(),
    )
    .unwrap();
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Root".into(),
        vec![],
        None,
        vec![requirement],
    );
    let mut output = BinaryOutput::with_capacity(2048);
    write(&mut output, &body).unwrap();
    let bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &24_u16.to_be_bytes());
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
    for version in [7_u16, 18, 23] {
        let mut old = bytes.clone();
        old[..2].copy_from_slice(&version.to_be_bytes());
        let mut budget = RecordDecodeAttempt::begin(
            Default::default(),
            old.len() as u64,
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap();
        assert_eq!(
            decode(&mut BinaryInput::new(&old), &mut budget)
                .unwrap_err()
                .kind(),
            Kind::UnsupportedRecordVariant
        );
    }
}
