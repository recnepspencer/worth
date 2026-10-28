use worth_foundational::{
    compare_canonical_basis, prepare_canonical_basis_sequence, prepare_canonical_comparison,
    CanonicalBasisDomain, CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus,
    CanonicalBasisReadyArtifact, CanonicalBasisValue, CanonicalComparisonOutcome,
    CanonicalComparisonReadinessProofs, CanonicalEquivalenceBasis, CanonicalIntegerWidth,
    CanonicalMismatchKind, CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;

fn version(name: &str) -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new(name).expect("valid canonicalization version")
}

fn state_entry(name: &str, value: i64) -> CanonicalBasisEntry {
    entry(
        CanonicalBasisDomain::AuthoritativeState,
        name,
        CanonicalBasisEntryKind::Value,
        value,
    )
}

fn value_entry(name: &str, value: i64) -> CanonicalBasisEntry {
    entry(
        CanonicalBasisDomain::Value,
        name,
        CanonicalBasisEntryKind::Value,
        value,
    )
}

fn entry(
    domain: CanonicalBasisDomain,
    name: &str,
    kind: CanonicalBasisEntryKind,
    value: i64,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        domain,
        CanonicalBasisLocus::Named(name.into()),
        kind,
        CanonicalBasisValue::SignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: i128::from(value),
        },
    )
}

fn ready(
    version: CanonicalizationRuleVersion,
    domain: CanonicalBasisDomain,
    entries: impl IntoIterator<Item = CanonicalBasisEntry>,
) -> CanonicalBasisReadyArtifact {
    match prepare_canonical_basis_sequence(version, domain, entries) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("basis should be ready"),
    }
}

fn compare(
    equivalence_basis: CanonicalEquivalenceBasis,
    left: CanonicalBasisReadyArtifact,
    right: CanonicalBasisReadyArtifact,
) -> CanonicalComparisonOutcome {
    let ready = match prepare_canonical_comparison(equivalence_basis, left, right) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("comparison should be ready"),
    };
    compare_canonical_basis(&ready)
}

fn accepts_comparison_readiness_proofs(_: &CanonicalComparisonReadinessProofs) {}

#[test]
fn exact_comparison_requires_ready_input_and_reports_equivalence_basis() {
    let left = ready(
        version("m2.phase3"),
        CanonicalBasisDomain::Value,
        [value_entry("alpha", 1), value_entry("zeta", 2)],
    );
    let right = ready(
        version("m2.phase3"),
        CanonicalBasisDomain::Value,
        [value_entry("zeta", 2), value_entry("alpha", 1)],
    );
    let comparison_ready = match prepare_canonical_comparison(
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left,
        right,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("comparison should be ready"),
    };

    accepts_comparison_readiness_proofs(comparison_ready.proofs());
    match compare_canonical_basis(&comparison_ready) {
        CanonicalComparisonOutcome::Equivalent(equivalent) => {
            assert_eq!(
                equivalent.equivalence_basis(),
                CanonicalEquivalenceBasis::ExactCanonicalBasis
            );
            assert_eq!(equivalent.domain(), CanonicalBasisDomain::Value);
            assert_eq!(equivalent.entry_count(), 2);
        }
        _ => panic!("same canonical entries should compare equivalent"),
    }
}

#[test]
fn compatibility_lowered_native_equivalence_is_explicit_not_inferred() {
    let left = ready(
        version("m2.phase3.compat"),
        CanonicalBasisDomain::AuthoritativeState,
        [state_entry("task.summary.title", 1)],
    );
    let right = ready(
        version("m2.phase3.compat"),
        CanonicalBasisDomain::AuthoritativeState,
        [state_entry("task.summary.title", 1)],
    );

    match compare(
        CanonicalEquivalenceBasis::CompatibilityLoweredNativeEquivalence,
        left,
        right,
    ) {
        CanonicalComparisonOutcome::Equivalent(equivalent) => assert_eq!(
            equivalent.equivalence_basis(),
            CanonicalEquivalenceBasis::CompatibilityLoweredNativeEquivalence
        ),
        _ => panic!("compatibility-lowered native equivalence should be admitted explicitly"),
    }
}

#[test]
fn comparison_reports_smallest_value_mismatch_locus_for_blind_consumers() {
    let left = ready(
        version("m2.phase3.value"),
        CanonicalBasisDomain::Value,
        [value_entry("same", 1)],
    );
    let right = ready(
        version("m2.phase3.value"),
        CanonicalBasisDomain::Value,
        [value_entry("same", 2)],
    );

    match compare(CanonicalEquivalenceBasis::ExactCanonicalBasis, left, right) {
        CanonicalComparisonOutcome::Mismatched(mismatch) => {
            assert_eq!(mismatch.kind(), CanonicalMismatchKind::ValueMismatch);
            assert_eq!(
                mismatch.left_locus(),
                Some(&CanonicalBasisLocus::Named("same".into()))
            );
            assert_eq!(
                mismatch.right_locus(),
                Some(&CanonicalBasisLocus::Named("same".into()))
            );
            assert_eq!(mismatch.left_domain(), CanonicalBasisDomain::Value);
            assert_eq!(mismatch.right_domain(), CanonicalBasisDomain::Value);
            assert_eq!(
                mismatch.left_entry_kind(),
                Some(CanonicalBasisEntryKind::Value)
            );
            assert_eq!(
                mismatch.right_entry_kind(),
                Some(CanonicalBasisEntryKind::Value)
            );
        }
        _ => panic!("different values should report value mismatch"),
    }
}

#[test]
fn comparison_distinguishes_missing_additional_and_entry_kind_mismatch() {
    let additional_left = ready(
        version("m2.phase3.additional"),
        CanonicalBasisDomain::Value,
        [value_entry("left.only", 1), value_entry("shared", 2)],
    );
    let additional_right = ready(
        version("m2.phase3.additional"),
        CanonicalBasisDomain::Value,
        [value_entry("shared", 2)],
    );
    match compare(
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        additional_left,
        additional_right,
    ) {
        CanonicalComparisonOutcome::Mismatched(mismatch) => {
            assert_eq!(mismatch.kind(), CanonicalMismatchKind::AdditionalEntry);
            assert_eq!(
                mismatch.left_locus(),
                Some(&CanonicalBasisLocus::Named("left.only".into()))
            );
            assert_eq!(mismatch.right_locus(), None);
        }
        _ => panic!("left-only entry should be additional"),
    }

    let missing_left = ready(
        version("m2.phase3.missing"),
        CanonicalBasisDomain::Value,
        [value_entry("shared", 2)],
    );
    let missing_right = ready(
        version("m2.phase3.missing"),
        CanonicalBasisDomain::Value,
        [value_entry("right.only", 1), value_entry("shared", 2)],
    );
    match compare(
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        missing_left,
        missing_right,
    ) {
        CanonicalComparisonOutcome::Mismatched(mismatch) => {
            assert_eq!(mismatch.kind(), CanonicalMismatchKind::MissingEntry);
            assert_eq!(mismatch.left_locus(), None);
            assert_eq!(
                mismatch.right_locus(),
                Some(&CanonicalBasisLocus::Named("right.only".into()))
            );
        }
        _ => panic!("right-only entry should be missing"),
    }

    let kind_left = ready(
        version("m2.phase3.kind"),
        CanonicalBasisDomain::Value,
        [entry(
            CanonicalBasisDomain::Value,
            "same-locus",
            CanonicalBasisEntryKind::Value,
            1,
        )],
    );
    let kind_right = ready(
        version("m2.phase3.kind"),
        CanonicalBasisDomain::Value,
        [entry(
            CanonicalBasisDomain::Value,
            "same-locus",
            CanonicalBasisEntryKind::Header,
            1,
        )],
    );
    match compare(
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        kind_left,
        kind_right,
    ) {
        CanonicalComparisonOutcome::Mismatched(mismatch) => {
            assert_eq!(mismatch.kind(), CanonicalMismatchKind::EntryKindMismatch);
            assert_eq!(
                mismatch.left_entry_kind(),
                Some(CanonicalBasisEntryKind::Value)
            );
            assert_eq!(
                mismatch.right_entry_kind(),
                Some(CanonicalBasisEntryKind::Header)
            );
        }
        _ => panic!("same locus with different entry kinds should be kind mismatch"),
    }
}

#[test]
fn unsupported_comparison_posture_is_structured_for_versions_domains_and_scopes() {
    let left_version = ready(
        version("m2.phase3.v1"),
        CanonicalBasisDomain::Value,
        [value_entry("same", 1)],
    );
    let right_version = ready(
        version("m2.phase3.v2"),
        CanonicalBasisDomain::Value,
        [value_entry("same", 1)],
    );
    match compare(
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left_version,
        right_version,
    ) {
        CanonicalComparisonOutcome::Unsupported(mismatch) => {
            assert_eq!(mismatch.kind(), CanonicalMismatchKind::VersionMismatch);
            assert_eq!(
                mismatch.equivalence_basis(),
                CanonicalEquivalenceBasis::ExactCanonicalBasis
            );
        }
        _ => panic!("version mismatch should fail closed as unsupported"),
    }

    let left_domain = ready(
        version("m2.phase3.domain"),
        CanonicalBasisDomain::Value,
        [value_entry("same", 1)],
    );
    let right_domain = ready(
        version("m2.phase3.domain"),
        CanonicalBasisDomain::Identity,
        [entry(
            CanonicalBasisDomain::Identity,
            "same",
            CanonicalBasisEntryKind::Identity,
            1,
        )],
    );
    match compare(
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left_domain,
        right_domain,
    ) {
        CanonicalComparisonOutcome::Unsupported(mismatch) => {
            assert_eq!(
                mismatch.kind(),
                CanonicalMismatchKind::UnsupportedComparison
            );
            assert_eq!(mismatch.left_domain(), CanonicalBasisDomain::Value);
            assert_eq!(mismatch.right_domain(), CanonicalBasisDomain::Identity);
        }
        _ => panic!("cross-domain exact comparison should be unsupported"),
    }

    let projection_left = ready(
        version("m2.phase3.projection"),
        CanonicalBasisDomain::Value,
        [value_entry("same", 1)],
    );
    let projection_right = ready(
        version("m2.phase3.projection"),
        CanonicalBasisDomain::Value,
        [value_entry("same", 1)],
    );
    match compare(
        CanonicalEquivalenceBasis::ProjectionEquivalent,
        projection_left,
        projection_right,
    ) {
        CanonicalComparisonOutcome::Unsupported(mismatch) => {
            assert_eq!(
                mismatch.kind(),
                CanonicalMismatchKind::UnsupportedComparison
            );
            assert_eq!(
                mismatch.equivalence_basis(),
                CanonicalEquivalenceBasis::ProjectionEquivalent
            );
        }
        _ => panic!("projection equivalence should not be silently inferred"),
    }
}
