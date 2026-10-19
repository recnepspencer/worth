//! V26 encodes effect meaning; older profiles cannot silently consume the extension.
use super::*;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_declaration::facade::application_schema::{
    ApplicationExternalEffectProtocol, ApplicationFieldPresence,
    ApplicationOperationBodyInputEmission as Emission,
    ApplicationOperationBodyRequirementExpression as Expression,
    ApplicationOperationBodyRequirementOperand as Operand,
    ApplicationOperationBodyRequirementProgram as Requirement,
    ApplicationOperationBodyRequirementSource as Source,
};

#[test]
fn whole_input_emission_v26_roundtrips_conditional_and_unconditional_meaning() {
    let expression =
        worth_foundational::expression_api::expressions().reading_within(Expression::PROFILE);
    let condition = Requirement::from_untrusted(
        "notify".into(),
        vec![Operand::from_untrusted(
            "ready".into(),
            Source::Field {
                field: ApplicationFieldBindingLocus::new("Item", "Facts", "Ready"),
                binding: WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.bool".into()),
                scalar_family: ScalarAspectType::Bool,
                presence: ApplicationFieldPresence::Required,
                presence_only: false,
            },
        )],
        None,
        expression.parse("ready").unwrap().encode(),
        "ready".into(),
    )
    .unwrap();
    for condition in [None, Some(condition)] {
        let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
            "Item".into(),
            vec![],
            None,
            vec![],
        )
        .with_untrusted_input_emission(Some(Emission {
            effect: "Notice".into(),
            input_binding: WorthQueryPortableTypeIdentity::from_untrusted("item.input".into()),
            payload_binding: WorthQueryPortableTypeIdentity::from_untrusted("item.notice".into()),
            protocol: ApplicationExternalEffectProtocol::new(
                BoundaryProtocolIdentity::new("item.notice"),
                BoundaryProtocolVersion::new(3),
            ),
            maximum_payload_bytes: 512,
            condition,
        }));
        let mut output = BinaryOutput::with_capacity(2048);
        write(&mut output, &body).unwrap();
        let mut bytes = output.into_bytes();
        assert_eq!(&bytes[..2], &26_u16.to_be_bytes());
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
        bytes[..2].copy_from_slice(&25_u16.to_be_bytes());
        let mut budget = RecordDecodeAttempt::begin(
            Default::default(),
            bytes.len() as u64,
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap();
        let mut input = BinaryInput::new(&bytes);
        let old = decode(&mut input, &mut budget).unwrap();
        assert!(old.input_emission().is_none());
        // Outer record admission requires exact consumption, so this is rejected.
        assert!(!input.is_finished());
    }
}
