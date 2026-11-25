//! Omitted-reversal grants persist under their own mechanism tag and refuse
//! nesting or an empty grant rather than read as a full-coverage inverse.
use super::{decode_member, encode_member, try_decode_member};
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_declaration::facade::{
    application_aftermath::*, application_schema::ApplicationSchemaMember,
};

fn locus(field: &str) -> PortablePreImageLocus {
    PortablePreImageLocus::from_untrusted_fields(
        ARCHIVE_DECODE,
        "Task".into(),
        "Facts".into(),
        field.into(),
    )
}

fn member(omitted: Vec<PortablePreImageLocus>) -> ApplicationSchemaMember {
    let demand =
        PortablePreImageDemand::from_untrusted_fields(ARCHIVE_DECODE, vec![locus("title")], 64)
            .with_untrusted_omitted_reversals(ARCHIVE_DECODE, omitted);
    let inverse = PortableRecordedInverse::from_untrusted_fields(
        ARCHIVE_DECODE,
        "RestoreTask".into(),
        DeclaredLoweringCorrespondenceRef::new("task-edit.v1"),
        DeclaredAftermathPostcondition::ExactPriorTruth,
        demand,
    );
    ApplicationSchemaMember::OperationAftermath {
        operation: "EditTask".into(),
        contract: PortableApplicationAftermathContract::from_untrusted_fields(
            ARCHIVE_DECODE,
            DeclaredCorrectionAuthority::RuntimeAlone,
            Some(PortableCorrectionMechanism::RecordedInverse(inverse)),
            None,
        ),
    }
}

#[test]
fn an_omitted_reversal_grant_roundtrips_and_an_empty_grant_refuses() {
    let granted = member(vec![locus("revision")]);
    let bytes = encode_member(&granted);
    assert_eq!(decode_member(&bytes), granted);
    let plain = encode_member(&member(vec![]));
    assert_ne!(bytes, plain, "a grant is never encoded as full coverage");
    assert_eq!(decode_member(&plain), member(vec![]));
    // The grant sequence ends the record; an empty one is not a grant.
    let suffix = b"\0\0\0\x01\0\0\0\x04Task\0\0\0\x05Facts\0\0\0\x08revision";
    let at = bytes
        .windows(suffix.len())
        .position(|window| window == suffix)
        .expect("the grant follows the inner mechanism");
    let mut empty = bytes[..at].to_vec();
    empty.extend_from_slice(&[0, 0, 0, 0]);
    empty.extend_from_slice(&bytes[at + suffix.len()..]);
    assert_eq!(
        try_decode_member(&empty).unwrap_err().kind(),
        crate::facade::WorthQueryPackageArchiveDenialKind::InvalidRecordShape
    );
}
