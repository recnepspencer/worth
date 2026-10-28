use worth_foundational::{
    compare_canonical_basis, prepare_canonical_basis_sequence, prepare_canonical_comparison,
    CanonicalBasisDomain, CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus,
    CanonicalBasisReadyArtifact, CanonicalBasisValue, CanonicalComparisonOutcome,
    CanonicalEquivalenceBasis, CanonicalIntegerWidth, CanonicalMismatchKind,
    CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;

fn version(name: &str) -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new(name).expect("valid canonicalization version")
}

fn value_entry(name: &str, value: i64) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Value,
        CanonicalBasisLocus::Named(name.into()),
        CanonicalBasisEntryKind::Value,
        CanonicalBasisValue::SignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: i128::from(value),
        },
    )
}

fn ready(entries: impl IntoIterator<Item = CanonicalBasisEntry>) -> CanonicalBasisReadyArtifact {
    match prepare_canonical_basis_sequence(
        version("m2.golden.comparison"),
        CanonicalBasisDomain::Value,
        entries,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("basis should be ready"),
    }
}

fn compare(
    left: CanonicalBasisReadyArtifact,
    right: CanonicalBasisReadyArtifact,
) -> CanonicalComparisonOutcome {
    let ready = match prepare_canonical_comparison(
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left,
        right,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("comparison should be ready"),
    };

    compare_canonical_basis(&ready)
}

#[test]
fn equivalence_golden_artifact_fixes_exact_basis_semantics_for_reordered_entries() {
    match compare(
        ready([value_entry("zeta", 2), value_entry("alpha", 1)]),
        ready([value_entry("alpha", 1), value_entry("zeta", 2)]),
    ) {
        CanonicalComparisonOutcome::Equivalent(equivalent) => {
            assert_eq!(
                equivalent.equivalence_basis(),
                CanonicalEquivalenceBasis::ExactCanonicalBasis
            );
            assert_eq!(equivalent.domain(), CanonicalBasisDomain::Value);
            assert_eq!(equivalent.entry_count(), 2);
        }
        other => panic!("golden equivalence fixture changed unexpectedly: {other:?}"),
    }
}

#[test]
fn mismatch_golden_artifact_fixes_first_locus_and_kind_semantics() {
    match compare(
        ready([
            value_entry("alpha", 1),
            value_entry("beta", 20),
            value_entry("gamma", 3),
        ]),
        ready([
            value_entry("alpha", 1),
            value_entry("beta", 21),
            value_entry("gamma", 4),
        ]),
    ) {
        CanonicalComparisonOutcome::Mismatched(mismatch) => {
            assert_eq!(mismatch.kind(), CanonicalMismatchKind::ValueMismatch);
            assert_eq!(
                mismatch.left_locus(),
                Some(&CanonicalBasisLocus::Named("beta".into()))
            );
            assert_eq!(
                mismatch.right_locus(),
                Some(&CanonicalBasisLocus::Named("beta".into()))
            );
            assert_eq!(mismatch.left_domain(), CanonicalBasisDomain::Value);
            assert_eq!(mismatch.right_domain(), CanonicalBasisDomain::Value);
            assert_eq!(
                mismatch.equivalence_basis(),
                CanonicalEquivalenceBasis::ExactCanonicalBasis
            );
        }
        other => panic!("golden mismatch fixture changed unexpectedly: {other:?}"),
    }
}
