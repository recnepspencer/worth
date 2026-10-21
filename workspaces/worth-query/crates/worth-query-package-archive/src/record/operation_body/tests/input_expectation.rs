//! New guard bytes cannot be interpreted as an old assignment or literal guard.
use super::*;
#[test]
fn input_expectation_v30_roundtrips_and_cannot_downgrade_to_v29() {
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Item".into(),
        vec![
            ApplicationOperationBodyScopeField::ExpectInput {
                field: ApplicationFieldBindingLocus::new("Item", "Facts", "Company"),
                binding: WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.string".into()),
                scalar_family: ScalarAspectType::String,
                member: "company".into(),
            },
            ApplicationOperationBodyScopeField::Assign {
                field: ApplicationFieldBindingLocus::new("Item", "Facts", "Ready"),
                binding: WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.bool".into()),
                value: AspectValue::Bool(true),
            },
        ],
        None,
        vec![],
    );
    let mut output = BinaryOutput::with_capacity(1024);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &30_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let mut input = BinaryInput::new(&bytes);
    assert_eq!(decode(&mut input, &mut budget).unwrap(), body);
    assert!(input.is_finished());
    bytes[..2].copy_from_slice(&29_u16.to_be_bytes());
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
