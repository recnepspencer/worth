#[allow(unused_imports)] use crate::certification::ordinary_context::canonicalization::golden_artifacts::equivalence_and_mismatch::*;
use {worth_foundational::compare_canonical_basis, worth_foundational::prepare_canonical_basis_sequence, worth_foundational::prepare_canonical_comparison, worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisReadyArtifact, worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalComparisonOutcome, worth_foundational::CanonicalEquivalenceBasis, worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalMismatchKind, worth_foundational::CanonicalizationRuleVersion};
use {worth_proof::TransitionOutcome};

#[test]
fn equivalence_golden_artifact_fixes_exact_basis_semantics_for_reordered_entries() {
    match compare(
        ready([value_entry("zeta", 2), value_entry("alpha", 1)]),
        ready([value_entry("alpha", 1), value_entry("zeta", 2)]),
    ) {
        worth_foundational::CanonicalComparisonOutcome::Equivalent(equivalent) => {
            assert_eq!(
                equivalent.equivalence_basis(),
                worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis
            );
            assert_eq!(
                equivalent.domain(),
                worth_foundational::CanonicalBasisDomain::Value
            );
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
        worth_foundational::CanonicalComparisonOutcome::Mismatched(mismatch) => {
            assert_eq!(
                mismatch.kind(),
                worth_foundational::CanonicalMismatchKind::ValueMismatch
            );
            assert_eq!(
                mismatch.left_locus(),
                Some(&worth_foundational::CanonicalBasisLocus::Named(
                    "beta".into()
                ))
            );
            assert_eq!(
                mismatch.right_locus(),
                Some(&worth_foundational::CanonicalBasisLocus::Named(
                    "beta".into()
                ))
            );
            assert_eq!(
                mismatch.left_domain(),
                worth_foundational::CanonicalBasisDomain::Value
            );
            assert_eq!(
                mismatch.right_domain(),
                worth_foundational::CanonicalBasisDomain::Value
            );
            assert_eq!(
                mismatch.equivalence_basis(),
                worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis
            );
        }
        other => panic!("golden mismatch fixture changed unexpectedly: {other:?}"),
    }
}
