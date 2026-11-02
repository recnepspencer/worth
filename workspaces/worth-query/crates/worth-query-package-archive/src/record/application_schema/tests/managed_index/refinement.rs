//! Refinement metadata survives the real archive; no callback/root is serialized.
use super::*;
#[test]
fn row_local_refinement_member_round_trip_preserves_meaning_and_legacy_absence() {
    let old = definition();
    assert!(old.refinement().is_none());
    let mut parts = old.parts().clone();
    parts.semantic_version = 4;
    parts.structure = ApplicationManagedIndexSourceStructure::BinarySourceTree;
    parts.membership = ApplicationManagedIndexMembership::CheckedSummarySourceTree;
    assert_eq!(
        ErasedApplicationManagedIndexDefinition::from_untrusted_parts(parts.clone()).validate(),
        Err(ApplicationManagedIndexDefinitionDenial::UnsupportedRefinement)
    );
    let refined = ErasedApplicationManagedIndexDefinition::from_refinement_parts(
        parts.clone(),
        ApplicationManagedIndexRefinement::row_local("domain.exact-name.v1", 7).meaning(),
    );
    let member = ApplicationSchemaMember::ManagedIndex {
        definition: refined,
    };
    let bytes = encode_member(&member);
    assert_eq!(&bytes[..4], &[0, 48, 0, 4]);
    let restored = decode_member(&bytes);
    assert_eq!(restored, member);
    assert_eq!(encode_member(&restored), bytes);
    assert_eq!(
        try_decode_member(&bytes[..bytes.len() - 1])
            .unwrap_err()
            .kind(),
        Kind::Truncated
    );
    let changed = ApplicationSchemaMember::ManagedIndex {
        definition: ErasedApplicationManagedIndexDefinition::from_refinement_parts(
            parts,
            ApplicationManagedIndexRefinement::row_local("domain.exact-name.v1", 8).meaning(),
        ),
    };
    assert_ne!(encode_member(&changed), bytes);
    assert_eq!(
        try_decode_member(&[0, 48, 0, 6]).unwrap_err().kind(),
        Kind::UnsupportedRecordVariant
    );
}
