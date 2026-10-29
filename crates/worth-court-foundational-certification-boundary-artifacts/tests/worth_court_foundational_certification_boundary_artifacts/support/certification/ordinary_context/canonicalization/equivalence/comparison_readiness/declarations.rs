pub(crate) use {worth_foundational::compare_canonical_basis, worth_foundational::prepare_canonical_basis_sequence, worth_foundational::prepare_canonical_comparison, worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisReadyArtifact, worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalComparisonOutcome, worth_foundational::CanonicalComparisonReadinessProofs, worth_foundational::CanonicalEquivalenceBasis, worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalMismatchKind, worth_foundational::CanonicalizationRuleVersion};
pub(crate) use {worth_proof::TransitionOutcome};

pub(crate) fn version(name: &str) -> CanonicalizationRuleVersion {
    worth_foundational::CanonicalizationRuleVersion::new(name).expect("valid canonicalization version")
}

pub(crate) fn state_entry(name: &str, value: i64) -> CanonicalBasisEntry {
    entry(
        worth_foundational::CanonicalBasisDomain::AuthoritativeState,
        name,
        worth_foundational::CanonicalBasisEntryKind::Value,
        value,
    )
}

pub(crate) fn value_entry(name: &str, value: i64) -> CanonicalBasisEntry {
    entry(
        worth_foundational::CanonicalBasisDomain::Value,
        name,
        worth_foundational::CanonicalBasisEntryKind::Value,
        value,
    )
}

pub(crate) fn entry(
    domain: CanonicalBasisDomain,
    name: &str,
    kind: CanonicalBasisEntryKind,
    value: i64,
) -> CanonicalBasisEntry {
    worth_foundational::CanonicalBasisEntry::new(
        domain,
        worth_foundational::CanonicalBasisLocus::Named(name.into()),
        kind,
        worth_foundational::CanonicalBasisValue::SignedInteger {
            width: worth_foundational::CanonicalIntegerWidth::Bits64,
            value: i128::from(value),
        },
    )
}

pub(crate) fn ready(
    version: CanonicalizationRuleVersion,
    domain: CanonicalBasisDomain,
    entries: impl IntoIterator<Item = CanonicalBasisEntry>,
) -> CanonicalBasisReadyArtifact {
    match prepare_canonical_basis_sequence(version, domain, entries) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("basis should be ready"),
    }
}

pub(crate) fn compare(
    equivalence_basis: CanonicalEquivalenceBasis,
    left: CanonicalBasisReadyArtifact,
    right: CanonicalBasisReadyArtifact,
) -> CanonicalComparisonOutcome {
    let ready = match prepare_canonical_comparison(equivalence_basis, left, right) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("comparison should be ready"),
    };
    compare_canonical_basis(&ready)
}

pub(crate) fn accepts_comparison_readiness_proofs(_: &CanonicalComparisonReadinessProofs) {}