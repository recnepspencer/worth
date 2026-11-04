//! Frozen schema metadata bytes are independent of the production serializer.
use super::*;
const CONTENT_SCHEMA_HEX: &str = concat!(
    "00120008000000010000010c0000000d617263686976652e74657374730000000d4d696e696d616c536368656d61",
    "00000001000000000000000400345b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b",
    "0000001165786163742d73616d706c652d677269640000000100000000010000000001000000044974656d000200",
    "0000044974656d0000000753616d706c6573000000000919c104000000000000000100020035000000044974656d",
    "0000000753616d706c65730000000656616c756573000100180000001b776f7274682e6e61746976652e636f6e74",
    "656e742d7265662e763100000001000000005b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b",
    "5b5b5b5b",
);
fn reconstruct(bytes: &[u8]) -> WorthQueryPortablePackageReconstructionCandidate {
    let exported = schema_package().export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let manifest = decode_manifest_frame(
        &encode_manifest_frame(exported.manifest(), limits).unwrap(),
        limits,
    )
    .unwrap();
    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(limits);
    for view in exported.views() {
        let frame = if view.family() == Family::ApplicationSchema {
            bytes.to_vec()
        } else {
            encode_record_frame(view, limits).unwrap()
        };
        let (index, record) = decoder.decode_frame(&frame).unwrap().into_parts();
        reconstruction = reconstruction.push_record(index, record).unwrap();
    }
    reconstruction.close().unwrap()
}
#[test]
fn content_schema_frozen_format_and_field_reference_roundtrip_without_executable_authority() {
    let bytes = decode_hex(CONTENT_SCHEMA_HEX);
    let candidate = reconstruct(&bytes);
    let view = candidate
        .views()
        .find(|view| view.family() == Family::ApplicationSchema)
        .unwrap();
    assert_eq!(
        encode_record_frame(view, WorthQueryPackageArchiveLimits::DEFAULT).unwrap(),
        bytes
    );
    let WorthQueryPortablePackageRecord::ApplicationSchema(record) = view.record() else {
        panic!("schema kind changed");
    };
    let ApplicationSchemaMember::ContentFormat { declaration } = &record.members()[0] else {
        panic!("format was lost");
    };
    assert_eq!(declaration.identity, [91; 32]);
    assert_eq!(declaration.name, "exact-sample-grid");
    assert_eq!(declaration.version, 1);
    assert_eq!(declaration.maximum_frame_bytes, 16 * 1024 * 1024);
    let ApplicationSchemaMember::Field {
        content_format,
        canonical,
        ..
    } = &record.members()[3]
    else {
        panic!("field was lost");
    };
    assert_eq!(*content_format, Some([91; 32]));
    assert!(canonical.is_none());
    let meaning = validate_portable_application_schema_freshly(record.clone()).unwrap();
    let mut changed = bytes;
    let name = changed
        .windows(17)
        .position(|word| word == b"exact-sample-grid")
        .unwrap();
    changed[name] = b'o';
    let alternate = reconstruct(&changed);
    let WorthQueryPortablePackageRecord::ApplicationSchema(other) = alternate
        .records()
        .iter()
        .find(|record| record.family() == Family::ApplicationSchema)
        .unwrap()
    else {
        panic!("schema kind changed");
    };
    let other = validate_portable_application_schema_freshly(other.clone()).unwrap();
    assert_ne!(meaning.identity(), other.identity());
}
#[test]
fn content_schema_unknown_format_or_non_ascii_name_cannot_be_readmitted() {
    let bytes = decode_hex(CONTENT_SCHEMA_HEX);
    for mutate in [0, 1] {
        let mut changed = bytes.clone();
        if mutate == 0 {
            *changed.last_mut().unwrap() ^= 1;
        } else {
            let name = changed
                .windows(17)
                .position(|word| word == b"exact-sample-grid")
                .unwrap();
            changed[name..name + 2].copy_from_slice(&[0xc3, 0xa9]);
        }
        let candidate = reconstruct(&changed);
        let WorthQueryPortablePackageRecord::ApplicationSchema(record) = candidate
            .records()
            .iter()
            .find(|record| record.family() == Family::ApplicationSchema)
            .unwrap()
        else {
            panic!("schema kind changed");
        };
        assert!(validate_portable_application_schema_freshly(record.clone()).is_err());
    }
}
