use worth_foundational::{
    admit_canonical_sequence_digest_derivation_with_budget, derive_canonical_digest,
    prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestDerivationDenial, CanonicalDigestWorkBudget, CanonicalIntegerWidth,
    CanonicalSingleSequenceDigestAlgorithmSlot, CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;

fn version(name: &str) -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new(name).expect("valid version")
}

fn entry(label: &str, value: i64) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Value,
        CanonicalBasisLocus::Named(label.into()),
        CanonicalBasisEntryKind::Value,
        CanonicalBasisValue::SignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: i128::from(value),
        },
    )
}

#[test]
fn bounded_digest_admission_denies_entry_and_exact_encoded_byte_overflow() {
    let version = version("m2.phase5.bounded");
    let sequence = prepare_canonical_basis_sequence(
        version.clone(),
        CanonicalBasisDomain::Value,
        [entry("alpha", 1), entry("beta", 2)],
    )
    .into_result()
    .expect("basis should prepare");
    let slot = CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        CanonicalDigestAlgorithmId::sha256(),
        CanonicalBasisDomain::Value,
        version.clone(),
    );
    let entry_budget = CanonicalDigestWorkBudget::new(1, 4_096).expect("nonzero budget");
    assert!(matches!(
        admit_canonical_sequence_digest_derivation_with_budget(sequence, slot, entry_budget),
        TransitionOutcome::Denied(CanonicalDigestDerivationDenial::EntryLimitExceeded {
            maximum: 1,
            actual: 2
        })
    ));

    let sequence = prepare_canonical_basis_sequence(
        version.clone(),
        CanonicalBasisDomain::Value,
        [entry("alpha", 1)],
    )
    .into_result()
    .expect("basis should prepare");
    let slot = CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        CanonicalDigestAlgorithmId::sha256(),
        CanonicalBasisDomain::Value,
        version,
    );
    let byte_budget = CanonicalDigestWorkBudget::new(1, 1).expect("nonzero budget");
    assert!(matches!(
        admit_canonical_sequence_digest_derivation_with_budget(sequence, slot, byte_budget),
        TransitionOutcome::Denied(CanonicalDigestDerivationDenial::EncodedByteLimitExceeded {
            maximum: 1,
            ..
        })
    ));
}

#[test]
fn digest_work_separates_canonical_material_from_sha256_compression() {
    let version = version("m2.phase5.work-evidence");
    let sequence = prepare_canonical_basis_sequence(
        version.clone(),
        CanonicalBasisDomain::Value,
        [entry("alpha", 1)],
    )
    .into_result()
    .expect("basis should prepare");
    let slot = CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        CanonicalDigestAlgorithmId::sha256(),
        CanonicalBasisDomain::Value,
        version,
    );
    let budget = CanonicalDigestWorkBudget::new(1, 4_096).expect("nonzero budget");
    let ready = admit_canonical_sequence_digest_derivation_with_budget(sequence, slot, budget)
        .into_result()
        .expect("bounded digest should admit");
    let digest = derive_canonical_digest(ready);
    let work = digest.metadata().work();

    assert_eq!(work.canonical_entry_count(), 1);
    assert!(work.canonical_encoded_bytes() > 0);
    assert!(work.canonical_material_allocation_bytes() >= work.canonical_encoded_bytes());
    assert_eq!(work.sha256_input_bytes(), work.canonical_encoded_bytes());
    assert_eq!(
        work.sha256_compression_block_count(),
        (work.sha256_input_bytes() + 9).div_ceil(64)
    );
}
