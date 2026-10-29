#[allow(unused_imports)] use crate::certification::ordinary_context::canonicalization::digest_slots::bounded_work::*;
use {worth_foundational::admit_canonical_sequence_digest_derivation_with_budget, worth_foundational::derive_canonical_digest, worth_foundational::prepare_canonical_basis_sequence, worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalDigestAlgorithmId, worth_foundational::CanonicalDigestDerivationDenial, worth_foundational::CanonicalDigestWorkBudget, worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot, worth_foundational::CanonicalizationRuleVersion};
use {worth_proof::TransitionOutcome};

#[test]
fn bounded_digest_admission_denies_entry_and_exact_encoded_byte_overflow() {
    let version = version("m2.phase5.bounded");
    let sequence = prepare_canonical_basis_sequence(
        version.clone(),
        worth_foundational::CanonicalBasisDomain::Value,
        [entry("alpha", 1), entry("beta", 2)],
    )
    .into_result()
    .expect("basis should prepare");
    let slot = worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        worth_foundational::CanonicalDigestAlgorithmId::sha256(),
        worth_foundational::CanonicalBasisDomain::Value,
        version.clone(),
    );
    let entry_budget = worth_foundational::CanonicalDigestWorkBudget::new(1, 4_096).expect("nonzero budget");
    assert!(matches!(
        admit_canonical_sequence_digest_derivation_with_budget(sequence, slot, entry_budget),
        worth_proof::TransitionOutcome::Denied(worth_foundational::CanonicalDigestDerivationDenial::EntryLimitExceeded {
            maximum: 1,
            actual: 2
        })
    ));

    let sequence = prepare_canonical_basis_sequence(
        version.clone(),
        worth_foundational::CanonicalBasisDomain::Value,
        [entry("alpha", 1)],
    )
    .into_result()
    .expect("basis should prepare");
    let slot = worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        worth_foundational::CanonicalDigestAlgorithmId::sha256(),
        worth_foundational::CanonicalBasisDomain::Value,
        version,
    );
    let byte_budget = worth_foundational::CanonicalDigestWorkBudget::new(1, 1).expect("nonzero budget");
    assert!(matches!(
        admit_canonical_sequence_digest_derivation_with_budget(sequence, slot, byte_budget),
        worth_proof::TransitionOutcome::Denied(worth_foundational::CanonicalDigestDerivationDenial::EncodedByteLimitExceeded {
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
        worth_foundational::CanonicalBasisDomain::Value,
        [entry("alpha", 1)],
    )
    .into_result()
    .expect("basis should prepare");
    let slot = worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        worth_foundational::CanonicalDigestAlgorithmId::sha256(),
        worth_foundational::CanonicalBasisDomain::Value,
        version,
    );
    let budget = worth_foundational::CanonicalDigestWorkBudget::new(1, 4_096).expect("nonzero budget");
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
