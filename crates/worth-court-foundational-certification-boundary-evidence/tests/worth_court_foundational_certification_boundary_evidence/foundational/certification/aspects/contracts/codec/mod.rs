// A codec is native contract meaning even when its physical scalar is unchanged.
#[allow(unused_imports)]
use crate::certification::ordinary_context::aspects::contracts::codec::*;
use worth_foundational::facade::PortableAspectContract;

#[test]
fn codec_contract_is_in_the_native_basis_and_portable_roundtrip() {
    let first = contract(Some(worth_foundational::facade::CanonicalDigestId::new(
        [1; 32],
    )));
    let second = contract(Some(worth_foundational::facade::CanonicalDigestId::new(
        [2; 32],
    )));
    assert_ne!(material(first.clone()), material(second.clone()));
    assert_ne!(material(first.clone()), material(contract(None)));
    for original in [first, second, contract(None)] {
        let portable = worth_foundational::facade::PortableAspectContract::from_contract(&original);
        let bytes = serde_json::to_vec(&portable).unwrap();
        let restored: PortableAspectContract = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(restored.readmit().unwrap(), original);
    }
}

#[test]
fn codec_contract_change_is_not_ordinary_scalar_compatibility() {
    let first = contract(Some(worth_foundational::facade::CanonicalDigestId::new(
        [1; 32],
    )));
    assert_eq!(
        first.classify_evolution_to(&first).kind(),
        worth_foundational::facade::AspectEvolutionKind::Unchanged
    );
    for next in [
        contract(None),
        contract(Some(worth_foundational::facade::CanonicalDigestId::new(
            [2; 32],
        ))),
    ] {
        assert_eq!(
            first.classify_evolution_to(&next).kind(),
            worth_foundational::facade::AspectEvolutionKind::Incompatible
        );
        assert_eq!(
            next.classify_evolution_to(&first).kind(),
            worth_foundational::facade::AspectEvolutionKind::Incompatible
        );
    }
}
