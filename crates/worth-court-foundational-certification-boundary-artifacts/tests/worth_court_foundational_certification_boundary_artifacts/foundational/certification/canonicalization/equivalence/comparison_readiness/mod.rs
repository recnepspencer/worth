#[allow(unused_imports)]
use crate::certification::ordinary_context::canonicalization::equivalence::comparison_readiness::*;
use {
    worth_foundational::compare_canonical_basis, worth_foundational::prepare_canonical_comparison,
};

#[test]
fn exact_comparison_requires_ready_input_and_reports_equivalence_basis() {
    let left = ready(
        version("m2.phase3"),
        worth_foundational::CanonicalBasisDomain::Value,
        [value_entry("alpha", 1), value_entry("zeta", 2)],
    );
    let right = ready(
        version("m2.phase3"),
        worth_foundational::CanonicalBasisDomain::Value,
        [value_entry("zeta", 2), value_entry("alpha", 1)],
    );
    let comparison_ready = match prepare_canonical_comparison(
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left,
        right,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("comparison should be ready"),
    };

    accepts_comparison_readiness_proofs(comparison_ready.proofs());
    match compare_canonical_basis(&comparison_ready) {
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
        _ => panic!("same canonical entries should compare equivalent"),
    }
}

#[test]
fn compatibility_lowered_native_equivalence_is_explicit_not_inferred() {
    let left = ready(
        version("m2.phase3.compat"),
        worth_foundational::CanonicalBasisDomain::AuthoritativeState,
        [state_entry("task.summary.title", 1)],
    );
    let right = ready(
        version("m2.phase3.compat"),
        worth_foundational::CanonicalBasisDomain::AuthoritativeState,
        [state_entry("task.summary.title", 1)],
    );

    match compare(
        worth_foundational::CanonicalEquivalenceBasis::CompatibilityLoweredNativeEquivalence,
        left,
        right,
    ) {
        worth_foundational::CanonicalComparisonOutcome::Equivalent(equivalent) => assert_eq!(
            equivalent.equivalence_basis(),
            worth_foundational::CanonicalEquivalenceBasis::CompatibilityLoweredNativeEquivalence
        ),
        _ => panic!("compatibility-lowered native equivalence should be admitted explicitly"),
    }
}

#[test]
fn comparison_reports_smallest_value_mismatch_locus_for_blind_consumers() {
    let left = ready(
        version("m2.phase3.value"),
        worth_foundational::CanonicalBasisDomain::Value,
        [value_entry("same", 1)],
    );
    let right = ready(
        version("m2.phase3.value"),
        worth_foundational::CanonicalBasisDomain::Value,
        [value_entry("same", 2)],
    );

    match compare(
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left,
        right,
    ) {
        worth_foundational::CanonicalComparisonOutcome::Mismatched(mismatch) => {
            assert_eq!(
                mismatch.kind(),
                worth_foundational::CanonicalMismatchKind::ValueMismatch
            );
            assert_eq!(
                mismatch.left_locus(),
                Some(&worth_foundational::CanonicalBasisLocus::Named(
                    "same".into()
                ))
            );
            assert_eq!(
                mismatch.right_locus(),
                Some(&worth_foundational::CanonicalBasisLocus::Named(
                    "same".into()
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
                mismatch.left_entry_kind(),
                Some(worth_foundational::CanonicalBasisEntryKind::Value)
            );
            assert_eq!(
                mismatch.right_entry_kind(),
                Some(worth_foundational::CanonicalBasisEntryKind::Value)
            );
        }
        _ => panic!("different values should report value mismatch"),
    }
}

#[test]
fn comparison_distinguishes_missing_additional_and_entry_kind_mismatch() {
    let additional_left = ready(
        version("m2.phase3.additional"),
        worth_foundational::CanonicalBasisDomain::Value,
        [value_entry("left.only", 1), value_entry("shared", 2)],
    );
    let additional_right = ready(
        version("m2.phase3.additional"),
        worth_foundational::CanonicalBasisDomain::Value,
        [value_entry("shared", 2)],
    );
    match compare(
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        additional_left,
        additional_right,
    ) {
        worth_foundational::CanonicalComparisonOutcome::Mismatched(mismatch) => {
            assert_eq!(
                mismatch.kind(),
                worth_foundational::CanonicalMismatchKind::AdditionalEntry
            );
            assert_eq!(
                mismatch.left_locus(),
                Some(&worth_foundational::CanonicalBasisLocus::Named(
                    "left.only".into()
                ))
            );
            assert_eq!(mismatch.right_locus(), None);
        }
        _ => panic!("left-only entry should be additional"),
    }

    let missing_left = ready(
        version("m2.phase3.missing"),
        worth_foundational::CanonicalBasisDomain::Value,
        [value_entry("shared", 2)],
    );
    let missing_right = ready(
        version("m2.phase3.missing"),
        worth_foundational::CanonicalBasisDomain::Value,
        [value_entry("right.only", 1), value_entry("shared", 2)],
    );
    match compare(
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        missing_left,
        missing_right,
    ) {
        worth_foundational::CanonicalComparisonOutcome::Mismatched(mismatch) => {
            assert_eq!(
                mismatch.kind(),
                worth_foundational::CanonicalMismatchKind::MissingEntry
            );
            assert_eq!(mismatch.left_locus(), None);
            assert_eq!(
                mismatch.right_locus(),
                Some(&worth_foundational::CanonicalBasisLocus::Named(
                    "right.only".into()
                ))
            );
        }
        _ => panic!("right-only entry should be missing"),
    }

    let kind_left = ready(
        version("m2.phase3.kind"),
        worth_foundational::CanonicalBasisDomain::Value,
        [entry(
            worth_foundational::CanonicalBasisDomain::Value,
            "same-locus",
            worth_foundational::CanonicalBasisEntryKind::Value,
            1,
        )],
    );
    let kind_right = ready(
        version("m2.phase3.kind"),
        worth_foundational::CanonicalBasisDomain::Value,
        [entry(
            worth_foundational::CanonicalBasisDomain::Value,
            "same-locus",
            worth_foundational::CanonicalBasisEntryKind::Header,
            1,
        )],
    );
    match compare(
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        kind_left,
        kind_right,
    ) {
        worth_foundational::CanonicalComparisonOutcome::Mismatched(mismatch) => {
            assert_eq!(
                mismatch.kind(),
                worth_foundational::CanonicalMismatchKind::EntryKindMismatch
            );
            assert_eq!(
                mismatch.left_entry_kind(),
                Some(worth_foundational::CanonicalBasisEntryKind::Value)
            );
            assert_eq!(
                mismatch.right_entry_kind(),
                Some(worth_foundational::CanonicalBasisEntryKind::Header)
            );
        }
        _ => panic!("same locus with different entry kinds should be kind mismatch"),
    }
}

#[test]
fn unsupported_comparison_posture_is_structured_for_versions_domains_and_scopes() {
    let left_version = ready(
        version("m2.phase3.v1"),
        worth_foundational::CanonicalBasisDomain::Value,
        [value_entry("same", 1)],
    );
    let right_version = ready(
        version("m2.phase3.v2"),
        worth_foundational::CanonicalBasisDomain::Value,
        [value_entry("same", 1)],
    );
    match compare(
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left_version,
        right_version,
    ) {
        worth_foundational::CanonicalComparisonOutcome::Unsupported(mismatch) => {
            assert_eq!(
                mismatch.kind(),
                worth_foundational::CanonicalMismatchKind::VersionMismatch
            );
            assert_eq!(
                mismatch.equivalence_basis(),
                worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis
            );
        }
        _ => panic!("version mismatch should fail closed as unsupported"),
    }

    let left_domain = ready(
        version("m2.phase3.domain"),
        worth_foundational::CanonicalBasisDomain::Value,
        [value_entry("same", 1)],
    );
    let right_domain = ready(
        version("m2.phase3.domain"),
        worth_foundational::CanonicalBasisDomain::Identity,
        [entry(
            worth_foundational::CanonicalBasisDomain::Identity,
            "same",
            worth_foundational::CanonicalBasisEntryKind::Identity,
            1,
        )],
    );
    match compare(
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left_domain,
        right_domain,
    ) {
        worth_foundational::CanonicalComparisonOutcome::Unsupported(mismatch) => {
            assert_eq!(
                mismatch.kind(),
                worth_foundational::CanonicalMismatchKind::UnsupportedComparison
            );
            assert_eq!(
                mismatch.left_domain(),
                worth_foundational::CanonicalBasisDomain::Value
            );
            assert_eq!(
                mismatch.right_domain(),
                worth_foundational::CanonicalBasisDomain::Identity
            );
        }
        _ => panic!("cross-domain exact comparison should be unsupported"),
    }

    let projection_left = ready(
        version("m2.phase3.projection"),
        worth_foundational::CanonicalBasisDomain::Value,
        [value_entry("same", 1)],
    );
    let projection_right = ready(
        version("m2.phase3.projection"),
        worth_foundational::CanonicalBasisDomain::Value,
        [value_entry("same", 1)],
    );
    match compare(
        worth_foundational::CanonicalEquivalenceBasis::ProjectionEquivalent,
        projection_left,
        projection_right,
    ) {
        worth_foundational::CanonicalComparisonOutcome::Unsupported(mismatch) => {
            assert_eq!(
                mismatch.kind(),
                worth_foundational::CanonicalMismatchKind::UnsupportedComparison
            );
            assert_eq!(
                mismatch.equivalence_basis(),
                worth_foundational::CanonicalEquivalenceBasis::ProjectionEquivalent
            );
        }
        _ => panic!("projection equivalence should not be silently inferred"),
    }
}
