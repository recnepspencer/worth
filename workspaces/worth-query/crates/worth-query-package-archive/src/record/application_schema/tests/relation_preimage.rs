//! Persist exact relation axes and the current output-field selection meaning.
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
        DeclaredLoweringCorrespondenceRef::new("stage-splice.v1"),
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
    // Strict fields and relation-aware fields have distinct current tags.
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
}

#[test]
fn output_field_archives_keep_mode_and_exact_output_distinct_from_strict_capture() {
    for capture in [
        PreImageOutputFieldCapture::ChangedPreserved,
        PreImageOutputFieldCapture::CompleteRetired,
    ] {
        for relations in [
            vec![],
            vec![PortablePreImageRelationLocus::from_untrusted_fields(
                "PreviousStage".into(),
                "Stage".into(),
                "Stage".into(),
            )],
        ] {
            let ApplicationSchemaMember::OperationAftermath {
                operation,
                contract,
            } = member(relations)
            else {
                unreachable!()
            };
            let Some(PortableCorrectionMechanism::RecordedInverse(inverse)) = contract.mechanism()
            else {
                unreachable!()
            };
            let changed = PortableRecordedInverse::from_untrusted_fields(
                inverse.inverse_operation_slot().to_owned(),
                inverse.lowering_correspondence().clone(),
                inverse.postcondition().clone(),
                inverse
                    .preimage_demand()
                    .clone()
                    .with_untrusted_output_fields(
                        PortablePreImageFieldOutput::from_untrusted_fields(
                            "owner-role".into(),
                            "Stage".into(),
                            capture,
                        ),
                    ),
            );
            let original = ApplicationSchemaMember::OperationAftermath {
                operation,
                contract: PortableApplicationAftermathContract::from_untrusted_fields(
                    DeclaredCorrectionAuthority::RuntimeAlone,
                    Some(PortableCorrectionMechanism::RecordedInverse(changed)),
                    None,
                ),
            };
            let bytes = encode_member(&original);
            assert_eq!(decode_member(&bytes), original);
            assert_eq!(encode_member(&decode_member(&bytes)), bytes);
            let offset = bytes
                .windows(10)
                .position(|part| part == b"owner-role")
                .unwrap();
            let tag = u16::from_be_bytes(bytes[offset - 6..offset - 4].try_into().unwrap());
            assert!(
                match capture {
                    PreImageOutputFieldCapture::ChangedPreserved => matches!(tag, 4 | 5),
                    PreImageOutputFieldCapture::CompleteRetired => matches!(tag, 6 | 7),
                },
                "output selection cannot reuse another mode's tag"
            );
            let mut unknown = bytes;
            unknown[offset - 6..offset - 4].copy_from_slice(&8u16.to_be_bytes());
            assert_eq!(
                try_decode_member(&unknown).unwrap_err().kind(),
                crate::facade::WorthQueryPackageArchiveDenialKind::UnsupportedRecordVariant
            );
        }
    }
}
