//! Exact reuse carriage is not ordinary retiring replacement; no decoded value is authority.
use super::*;
#[test]
fn complete_content_reuse_v35_roundtrips_without_changing_old_replacement_bytes() {
    let creation = ApplicationOperationBodyCreation::from_untrusted(
        "Snapshot".into(),
        "snapshot".into(),
        vec![],
    );
    let ordinary = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Row".into(),
        vec![],
        Some(creation.clone()),
        vec![],
    );
    let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Row".into(),
        vec![],
        Some(creation.with_untrusted_reuse_current_if_unchanged(true)),
        vec![],
    );
    let mut output = BinaryOutput::with_capacity(2048);
    write(&mut output, &body).unwrap();
    let bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &35_u16.to_be_bytes());
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
    let mut measure = crate::binary_encoding::BinaryEncodingMeasure::default();
    write(&mut measure, &body).unwrap();
    assert_eq!(measure.nested_entries(), decoded.finish().nested_entries());
    let mut downgraded = bytes.clone();
    downgraded[..2].copy_from_slice(&34_u16.to_be_bytes());
    let mut old = BinaryInput::new(&downgraded);
    let description = decode(&mut old, &mut budget(&downgraded)).unwrap();
    assert!(!old.is_finished()); // Enclosing record rejects these trailing bytes.
    assert!(!description
        .creation()
        .unwrap()
        .reuses_current_if_unchanged());
    let mut old = BinaryOutput::with_capacity(2048);
    write(&mut old, &ordinary).unwrap();
    assert_eq!(&old.into_bytes()[..2], &8_u16.to_be_bytes());
    let mut malformed = bytes;
    *malformed.last_mut().unwrap() = 2;
    assert_eq!(
        decode(&mut BinaryInput::new(&malformed), &mut budget(&malformed))
            .unwrap_err()
            .kind(),
        Kind::InvalidBooleanEncoding
    );
}
