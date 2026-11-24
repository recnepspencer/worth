//! Codec compatibility compares opaque revision fingerprints, not counters.
use super::*;
use worth_foundational::facade::*;
fn binding(revision: u64, codec: u8) -> DeclaredAspectContractBinding {
    let field = FieldDeclaration::new(
        FieldKey::new("value").unwrap(),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .unwrap()
    .with_codec(CanonicalDigestId::new([codec; 32]));
    DeclaredAspectContractBinding {
        binding: AspectBinding::EntityField {
            field: FieldKey::new("record").unwrap(),
        },
        contract: AspectContract::struct_aspect(
            AspectKey::new("record").unwrap(),
            AspectIdentity(1),
            AspectContractRevision(revision),
            StructAspectShape::new([field]).unwrap(),
        ),
    }
}
#[test]
fn a_changed_codec_accepts_a_distinct_lower_fingerprint() {
    let mut changes = Vec::new();
    assert!(compare_binding(
        KindId(1),
        &binding(u64::MAX, 1),
        &binding(1, 2),
        &mut changes
    )
    .is_ok());
    assert_eq!(changes.len(), 1);
}
#[test]
fn a_changed_codec_refuses_the_same_fingerprint() {
    let mut changes = Vec::new();
    assert!(matches!(
        compare_binding(KindId(1), &binding(7, 1), &binding(7, 2), &mut changes),
        Err(CodecFieldComparisonStop::Contract(_))
    ));
    assert!(changes.is_empty());
}
#[test]
fn a_fingerprint_change_without_codec_meaning_still_refuses() {
    let mut changes = Vec::new();
    assert!(matches!(
        compare_binding(KindId(1), &binding(7, 1), &binding(8, 1), &mut changes),
        Err(CodecFieldComparisonStop::Contract(_))
    ));
    assert!(changes.is_empty());
}
