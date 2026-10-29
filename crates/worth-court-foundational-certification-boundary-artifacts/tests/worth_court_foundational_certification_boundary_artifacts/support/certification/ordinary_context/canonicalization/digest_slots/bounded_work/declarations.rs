pub(crate) use {worth_foundational::admit_canonical_sequence_digest_derivation_with_budget, worth_foundational::derive_canonical_digest, worth_foundational::prepare_canonical_basis_sequence, worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalDigestAlgorithmId, worth_foundational::CanonicalDigestDerivationDenial, worth_foundational::CanonicalDigestWorkBudget, worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalSingleSequenceDigestAlgorithmSlot, worth_foundational::CanonicalizationRuleVersion};
pub(crate) use {worth_proof::TransitionOutcome};

pub(crate) fn version(name: &str) -> CanonicalizationRuleVersion {
    worth_foundational::CanonicalizationRuleVersion::new(name).expect("valid version")
}

pub(crate) fn entry(label: &str, value: i64) -> CanonicalBasisEntry {
    worth_foundational::CanonicalBasisEntry::new(
        worth_foundational::CanonicalBasisDomain::Value,
        worth_foundational::CanonicalBasisLocus::Named(label.into()),
        worth_foundational::CanonicalBasisEntryKind::Value,
        worth_foundational::CanonicalBasisValue::SignedInteger {
            width: worth_foundational::CanonicalIntegerWidth::Bits64,
            value: i128::from(value),
        },
    )
}