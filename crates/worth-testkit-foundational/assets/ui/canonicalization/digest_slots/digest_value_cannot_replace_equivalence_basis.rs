use worth_foundational::{
    admit_canonical_sequence_digest_derivation, derive_canonical_digest,
    prepare_canonical_basis_sequence, prepare_canonical_comparison, CanonicalBasisDomain,
    CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue,
    CanonicalDigestAlgorithmId, CanonicalIntegerWidth, CanonicalSingleSequenceDigestAlgorithmSlot,
    CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;

fn ready_value_sequence(version: CanonicalizationRuleVersion) -> worth_foundational::CanonicalBasisReadyArtifact {
    let entry = CanonicalBasisEntry::new(
        CanonicalBasisDomain::Value,
        CanonicalBasisLocus::Named("value".into()),
        CanonicalBasisEntryKind::Value,
        CanonicalBasisValue::SignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: 1,
        },
    );
    match prepare_canonical_basis_sequence(version, CanonicalBasisDomain::Value, [entry]) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("basis should be ready"),
    }
}

fn main() {
    let version = CanonicalizationRuleVersion::new("m2.phase5.ui").expect("valid version");
    let ready = ready_value_sequence(version.clone());
    let slot = CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        CanonicalDigestAlgorithmId::sha256(),
        CanonicalBasisDomain::Value,
        version.clone(),
    );
    let derivation = match admit_canonical_sequence_digest_derivation(ready, slot) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("digest derivation should be ready"),
    };
    let digest = derive_canonical_digest(derivation);

    let _ = prepare_canonical_comparison(
        digest.value().clone(),
        ready_value_sequence(version.clone()),
        ready_value_sequence(version),
    );
}
