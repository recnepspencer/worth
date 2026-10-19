//! V9 carries optional source intent in both existing and created fields.
use super::*;

#[test]
fn optional_assignment_and_initializer_roundtrip_and_old_version_refuses_each_opcode() {
    let binding = WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.string".to_owned());
    let assignment = ApplicationOperationBodyProgram::from_untrusted_scope_fields(vec![
        ApplicationOperationBodyScopeField::AssignOptionalInput {
            field: ApplicationFieldBindingLocus::new("Item", "Facts", "Note"),
            binding: binding.clone(),
            scalar_family: ScalarAspectType::String,
            member: "note".to_owned(),
        },
    ]);
    let creation = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Tenant".to_owned(),
        vec![],
        Some(ApplicationOperationBodyCreation::from_untrusted(
            "Item".to_owned(),
            "item".to_owned(),
            vec![ApplicationOperationBodyCreatedField::OptionalInput {
                field: ApplicationFieldBindingLocus::new("Item", "Facts", "Note"),
                binding,
                scalar_family: ScalarAspectType::String,
                member: "note".to_owned(),
            }],
        )),
        vec![],
    );
    for body in [assignment, creation] {
        let mut output = BinaryOutput::with_capacity(256);
        write(&mut output, &body).unwrap();
        let mut bytes = output.into_bytes();
        assert_eq!(&bytes[..2], &9_u16.to_be_bytes());
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
        bytes[..2].copy_from_slice(&8_u16.to_be_bytes());
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
}
