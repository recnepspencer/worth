//! Recovery distinguishes a stale fingerprint from missing transition authority.
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
fn changed_codec_denial_treats_revision_hashes_as_opaque_and_preserves_refusal_precedence() {
    for (old, new, expected) in [
        (
            7,
            7,
            "recovered field codec meaning changed without a changed semantic aspect revision",
        ),
        (
            u64::MAX,
            1,
            "recovered derived codec requires authenticated native transition admission",
        ),
    ] {
        let previous = binding(old, 1);
        let next = binding(new, 2);
        let (AspectShape::Struct(previous_fields), AspectShape::Struct(next_fields)) =
            (previous.contract.shape(), next.contract.shape())
        else {
            panic!("the fixture declares scalar fields");
        };
        let error = codec_change_denial(
            &previous.contract,
            &next.contract,
            &previous_fields.fields()[0],
            &next_fields.fields()[0],
        );
        assert_eq!(error.detail, expected);
    }
}
