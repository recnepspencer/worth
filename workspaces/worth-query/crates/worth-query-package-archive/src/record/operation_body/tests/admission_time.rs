//! An admission-time initializer is V45 meaning: it roundtrips, and its bytes
//! cannot be read under an older profile as another initializer.
use super::*;
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;

#[test]
fn admission_time_initializer_v45_roundtrips_and_cannot_downgrade_to_v44() {
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        ARCHIVE_DECODE,
        "Company".to_owned(),
        vec![],
        Some(ApplicationOperationBodyCreation::from_untrusted(
            ARCHIVE_DECODE,
            "Request".to_owned(),
            "request".to_owned(),
            vec![ApplicationOperationBodyCreatedField::AdmissionTime {
                field: ApplicationFieldBindingLocus::new("Request", "Facts", "CreatedAt"),
                binding: WorthQueryPortableTypeIdentity::from_untrusted(
                    ARCHIVE_DECODE,
                    "worth.foundational.canonical_timestamp.v1".to_owned(),
                ),
            }],
        )),
        vec![],
    );
    let mut output = BinaryOutput::with_capacity(256);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &45_u16.to_be_bytes());
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
    bytes[..2].copy_from_slice(&44_u16.to_be_bytes());
    assert_eq!(
        decode(&mut BinaryInput::new(&bytes), &mut budget(&bytes))
            .unwrap_err()
            .kind(),
        Kind::UnsupportedRecordVariant
    );
}
