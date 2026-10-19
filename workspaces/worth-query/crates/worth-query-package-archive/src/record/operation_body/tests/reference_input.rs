//! The new reference profile charges its sequence and leaves older bodies untouched.
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyReferenceInput as Reference;
#[test]
fn existing_reference_v37_roundtrip_nested_measure_and_old_profile_boundary() {
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Row".into(),
        vec![],
        None,
        vec![],
    )
    .with_untrusted_reference_inputs(vec![Reference {
        relation: "Validator".into(),
        from: "Row".into(),
        to: "Principal".into(),
        field: ApplicationFieldBindingLocus::new("Principal", "Facts", "Identity"),
        binding: WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.u64".into()),
        scalar_family: worth_foundational::facade::ScalarAspectType::UInt64,
        member: "validator".into(),
        key: "validator".into(),
    }]);
    let mut output = BinaryOutput::with_capacity(1024);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &37_u16.to_be_bytes());
    let budget = |bytes: &[u8]| {
        RecordDecodeAttempt::begin(
            Default::default(),
            bytes.len() as u64,
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap()
    };
    let mut input = BinaryInput::new(&bytes);
    let mut decoded = budget(&bytes);
    assert_eq!(decode(&mut input, &mut decoded).unwrap(), body);
    assert!(input.is_finished());
    let mut measured = crate::binary_encoding::BinaryEncodingMeasure::default();
    write(&mut measured, &body).unwrap();
    assert_eq!(measured.nested_entries(), decoded.finish().nested_entries());
    bytes[..2].copy_from_slice(&36_u16.to_be_bytes());
    let mut input = BinaryInput::new(&bytes);
    let old = decode(&mut input, &mut budget(&bytes)).unwrap();
    assert!(old.reference_inputs().is_empty());
    assert!(!input.is_finished());
    // A truncated new member cannot be accepted as a clear or empty reference list.
    bytes[..2].copy_from_slice(&37_u16.to_be_bytes());
    bytes.pop();
    assert!(decode(&mut BinaryInput::new(&bytes), &mut budget(&bytes)).is_err());
}
