//! The scope-bearing record is not an older body with discarded scope bytes.
use super::*;

#[test]
fn creation_only_v8_roundtrips_with_scope_and_exact_framing() {
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Tenant".to_owned(),
        vec![],
        Some(ApplicationOperationBodyCreation::from_untrusted(
            "Item".to_owned(),
            "item".to_owned(),
            vec![ApplicationOperationBodyCreatedField::Input {
                field: ApplicationFieldBindingLocus::new("Item", "Facts", "Name"),
                binding: WorthQueryPortableTypeIdentity::from_untrusted(
                    "worth.rust.string".to_owned(),
                ),
                scalar_family: ScalarAspectType::String,
                member: "name".to_owned(),
            }],
        )),
        vec![],
    );
    let mut output = BinaryOutput::with_capacity(256);
    write(&mut output, &body).unwrap();
    let bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &8_u16.to_be_bytes());
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
}

#[test]
fn old_creation_record_cannot_acquire_the_creation_only_semantics() {
    // A v3 body: empty old scope steps, followed by a zero-field creation.
    // Even this authority-free description cannot silently adopt new meaning.
    let mut output = BinaryOutput::with_capacity(64);
    output.u16(3);
    output.u32(0);
    output.text("Item");
    output.text("item");
    output.u32(0);
    let bytes = output.into_bytes();
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
