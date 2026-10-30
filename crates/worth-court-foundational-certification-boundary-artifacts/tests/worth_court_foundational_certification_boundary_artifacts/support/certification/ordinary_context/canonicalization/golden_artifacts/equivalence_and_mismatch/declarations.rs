pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::compare_canonical_basis,
    worth_foundational::prepare_canonical_basis_sequence,
    worth_foundational::prepare_canonical_comparison, worth_foundational::CanonicalBasisDomain,
    worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind,
    worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisReadyArtifact,
    worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalComparisonOutcome,
    worth_foundational::CanonicalEquivalenceBasis, worth_foundational::CanonicalIntegerWidth,
    worth_foundational::CanonicalMismatchKind, worth_foundational::CanonicalizationRuleVersion,
};

pub(crate) fn version(name: &str) -> CanonicalizationRuleVersion {
    worth_foundational::CanonicalizationRuleVersion::new(name)
        .expect("valid canonicalization version")
}

pub(crate) fn value_entry(name: &str, value: i64) -> CanonicalBasisEntry {
    worth_foundational::CanonicalBasisEntry::new(
        worth_foundational::CanonicalBasisDomain::Value,
        worth_foundational::CanonicalBasisLocus::Named(name.into()),
        worth_foundational::CanonicalBasisEntryKind::Value,
        worth_foundational::CanonicalBasisValue::SignedInteger {
            width: worth_foundational::CanonicalIntegerWidth::Bits64,
            value: i128::from(value),
        },
    )
}

pub(crate) fn ready(
    entries: impl IntoIterator<Item = CanonicalBasisEntry>,
) -> CanonicalBasisReadyArtifact {
    match prepare_canonical_basis_sequence(
        version("m2.golden.comparison"),
        worth_foundational::CanonicalBasisDomain::Value,
        entries,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("basis should be ready"),
    }
}

pub(crate) fn compare(
    left: CanonicalBasisReadyArtifact,
    right: CanonicalBasisReadyArtifact,
) -> CanonicalComparisonOutcome {
    let ready = match prepare_canonical_comparison(
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left,
        right,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("comparison should be ready"),
    };

    compare_canonical_basis(&ready)
}
