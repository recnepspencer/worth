pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::prepare_canonical_basis_bundle,
    worth_foundational::prepare_canonical_basis_sequence,
    worth_foundational::CanonicalBasisConstructionDenial, worth_foundational::CanonicalBasisDomain,
    worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind,
    worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisReadinessProofs,
    worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalBundleReadinessProofs,
    worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalizationRuleVersion,
};

pub(crate) fn version(name: &str) -> CanonicalizationRuleVersion {
    worth_foundational::CanonicalizationRuleVersion::new(name).expect("test version is nonempty")
}

pub(crate) fn value_entry(label: &str, value: i64) -> CanonicalBasisEntry {
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

pub(crate) fn accepts_basis_readiness_proofs(_: &CanonicalBasisReadinessProofs) {}

pub(crate) fn accepts_bundle_readiness_proofs(_: &CanonicalBundleReadinessProofs) {}
