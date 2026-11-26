//! An optional-field input guard is V44 meaning: it roundtrips and its bytes
//! cannot be read under an older profile as another step.
use super::*;
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
#[test]
fn optional_input_guard_v44_roundtrips_and_cannot_downgrade_to_v43() {
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        ARCHIVE_DECODE,
        "Task".into(),
        vec![ApplicationOperationBodyScopeField::ExpectOptionalInput {
            field: ApplicationFieldBindingLocus::new("Task", "Facts", "Due"),
            binding: WorthQueryPortableTypeIdentity::from_untrusted(
                ARCHIVE_DECODE,
                "worth.rust.canonical-date".into(),
            ),
            scalar_family: ScalarAspectType::Date,
            member: "expected_due".into(),
        }],
        None,
        vec![],
    );
    let mut output = BinaryOutput::with_capacity(1024);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &44_u16.to_be_bytes());
    let budget = |bytes: &[u8]| {
        RecordDecodeAttempt::begin(
            Default::default(),
            bytes.len() as u64,
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap()
    };
    let mut input = BinaryInput::new(&bytes);
    assert_eq!(decode(&mut input, &mut budget(&bytes)).unwrap(), body);
    assert!(input.is_finished());
    bytes[..2].copy_from_slice(&43_u16.to_be_bytes());
    assert_eq!(
        decode(&mut BinaryInput::new(&bytes), &mut budget(&bytes))
            .unwrap_err()
            .kind(),
        Kind::UnsupportedRecordVariant
    );
}
