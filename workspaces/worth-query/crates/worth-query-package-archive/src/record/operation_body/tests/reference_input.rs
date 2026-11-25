//! The new reference profile charges its sequence and leaves older bodies untouched.
use super::*;
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyReferenceInput as Reference,
    ApplicationOperationBodyReferenceInputMode as Mode,
};
fn body(mode: Mode) -> ApplicationOperationBodyProgram {
    ApplicationOperationBodyProgram::from_untrusted_in_scope(
        ARCHIVE_DECODE,
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
        binding: WorthQueryPortableTypeIdentity::from_untrusted(
            ARCHIVE_DECODE,
            "worth.rust.u64".into(),
        ),
        scalar_family: worth_foundational::facade::ScalarAspectType::UInt64,
        member: "validator".into(),
        key: "validator".into(),
        mode,
    }])
}
fn budget(bytes: &[u8]) -> RecordDecodeAttempt {
    RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap()
}
#[test]
fn existing_reference_v37_roundtrip_nested_measure_and_old_profile_boundary() {
    let body = body(Mode::ReplaceNullable);
    let mut output = BinaryOutput::with_capacity(1024);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &37_u16.to_be_bytes());
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
#[test]
fn exact_toggle_v42_roundtrips_its_bool_member_and_refuses_older_profiles_and_unknown_modes() {
    let body = body(Mode::ToggleExact {
        linked_member: "linked".into(),
    });
    let mut output = BinaryOutput::with_capacity(1024);
    write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &42_u16.to_be_bytes());
    let mut decoded = budget(&bytes);
    let mut input = BinaryInput::new(&bytes);
    assert_eq!(decode(&mut input, &mut decoded).unwrap(), body);
    assert!(input.is_finished());
    let mut measured = crate::binary_encoding::BinaryEncodingMeasure::default();
    write(&mut measured, &body).unwrap();
    assert_eq!(measured.nested_entries(), decoded.finish().nested_entries());
    // A pre-v42 profile has no mode slot, so a toggle cannot be downgraded to replacement.
    let denied = super::super::reference_input::write(
        &mut BinaryOutput::with_capacity(256),
        body.reference_inputs(),
        41,
    )
    .unwrap_err();
    assert_eq!(denied.kind(), Kind::UnsupportedRecordVariant);
    // The mode tag immediately precedes the length-prefixed `linked` member text.
    let member = [&6_u32.to_be_bytes()[..], b"linked"].concat();
    let tag = bytes
        .windows(member.len())
        .position(|window| window == member)
        .unwrap()
        - 2;
    assert_eq!(&bytes[tag..tag + 2], &1_u16.to_be_bytes());
    bytes[tag..tag + 2].copy_from_slice(&7_u16.to_be_bytes());
    let refused = decode(&mut BinaryInput::new(&bytes), &mut budget(&bytes)).unwrap_err();
    assert_eq!(refused.kind(), Kind::UnsupportedRecordVariant);
}
