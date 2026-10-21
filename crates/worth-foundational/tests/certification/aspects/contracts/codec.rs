//! A codec is native contract meaning even when its physical scalar is unchanged.
use worth_foundational::facade::{
    aspects, canonical_basis_sequence_material, prepare_aspect_contract_for_canonical_basis,
    AspectContract, AspectContractRevision, AspectEvolutionKind, AspectIdentity, AspectKey,
    CanonicalDigestId, CanonicalizationRuleVersion, PortableAspectContract, ScalarAspectType,
    StructAspectShape,
};

fn contract(codec: Option<CanonicalDigestId>) -> AspectContract {
    let shape = aspects()
        .struct_fields()
        .required("payload", ScalarAspectType::String)
        .finish()
        .unwrap();
    let fields = shape.fields().iter().cloned().map(|field| match codec {
        Some(codec) => field.with_codec(codec),
        None => field,
    });
    aspects()
        .contract()
        .for_key(AspectKey::new("result").unwrap())
        .identified_by(AspectIdentity(23))
        .at_revision(AspectContractRevision(1))
        .struct_aspect(StructAspectShape::new(fields).unwrap())
}

fn material(contract: AspectContract) -> String {
    let ready = prepare_aspect_contract_for_canonical_basis(
        CanonicalizationRuleVersion::new("codec-contract-test.v1").unwrap(),
        contract,
    )
    .into_result()
    .unwrap();
    canonical_basis_sequence_material(ready.payload())
}

#[test]
fn codec_contract_is_in_the_native_basis_and_portable_roundtrip() {
    let first = contract(Some(CanonicalDigestId::new([1; 32])));
    let second = contract(Some(CanonicalDigestId::new([2; 32])));
    assert_ne!(material(first.clone()), material(second.clone()));
    assert_ne!(material(first.clone()), material(contract(None)));
    for original in [first, second, contract(None)] {
        let portable = PortableAspectContract::from_contract(&original);
        let bytes = serde_json::to_vec(&portable).unwrap();
        let restored: PortableAspectContract = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(restored.readmit().unwrap(), original);
    }
}

#[test]
fn codec_contract_change_is_not_ordinary_scalar_compatibility() {
    let first = contract(Some(CanonicalDigestId::new([1; 32])));
    assert_eq!(
        first.classify_evolution_to(&first).kind(),
        AspectEvolutionKind::Unchanged
    );
    for next in [
        contract(None),
        contract(Some(CanonicalDigestId::new([2; 32]))),
    ] {
        assert_eq!(
            first.classify_evolution_to(&next).kind(),
            AspectEvolutionKind::Incompatible
        );
        assert_eq!(
            next.classify_evolution_to(&first).kind(),
            AspectEvolutionKind::Incompatible
        );
    }
}
