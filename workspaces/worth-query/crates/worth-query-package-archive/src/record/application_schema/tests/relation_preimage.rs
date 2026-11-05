//! Persist the optional relation vocabulary without rewriting field-only bytes.
use super::{decode_member, encode_member, try_decode_member};
use worth_query_declaration::facade::{
    application_aftermath::*, application_schema::ApplicationSchemaMember,
};
fn member(relations: Vec<PortablePreImageRelationLocus>) -> ApplicationSchemaMember {
    let demand = PortablePreImageDemand::from_untrusted_fields(
        vec![PortablePreImageLocus::from_untrusted_fields(
            "Stage".into(),
            "Facts".into(),
            "key".into(),
        )],
        4096,
    )
    .with_untrusted_relations(relations);
    let inverse = PortableRecordedInverse::from_untrusted_fields(
        "RestoreStage".into(),
        DeclaredLoweringCorrespondenceRef::new("stage-splice.v1").unwrap(),
        DeclaredAftermathPostcondition::ExactPriorTruth,
        demand,
    );
    ApplicationSchemaMember::OperationAftermath {
        operation: "RetireStage".into(),
        contract: PortableApplicationAftermathContract::from_untrusted_fields(
            DeclaredCorrectionAuthority::RuntimeAlone,
            Some(PortableCorrectionMechanism::RecordedInverse(inverse)),
            None,
        ),
    }
}
#[test]
fn relation_preimage_archive_retains_kind_and_both_endpoint_axes() {
    let original = member(vec![PortablePreImageRelationLocus::from_untrusted_fields(
        "PreviousStage".into(),
        "Stage".into(),
        "Stage".into(),
    )]);
    let bytes = encode_member(&original);
    assert_eq!(decode_member(&bytes), original);
    assert_eq!(encode_member(&decode_member(&bytes)), bytes);
    let field_only = encode_member(&member(vec![]));
    // The old mechanism is tag 1; a relation-aware mechanism has tag 3.
    let slot = b"RestoreStage";
    let offset = bytes
        .windows(slot.len())
        .position(|window| window == slot)
        .unwrap();
    assert_eq!(&bytes[offset - 6..offset - 4], &3u16.to_be_bytes());
    assert_eq!(&field_only[offset - 6..offset - 4], &1u16.to_be_bytes());
    // Independent axes: exact source and target names follow the relation name.
    let suffix = b"\0\0\0\x01\0\0\0\x0dPreviousStage\0\0\0\x05Stage\0\0\0\x05Stage\0\0";
    assert!(bytes.ends_with(suffix));
    let sequence = bytes.len() - suffix.len();
    let mut empty = bytes[..sequence].to_vec();
    empty.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
    assert_eq!(
        try_decode_member(&empty).unwrap_err().kind(),
        crate::facade::WorthQueryPackageArchiveDenialKind::InvalidRecordShape
    );
    let mut legacy_payload = bytes[..sequence].to_vec();
    legacy_payload[offset - 6..offset - 4].copy_from_slice(&1u16.to_be_bytes());
    legacy_payload.extend_from_slice(&[0, 0]);
    assert_eq!(
        legacy_payload, field_only,
        "old recorded-inverse fields retain their exact wire encoding"
    );
}
