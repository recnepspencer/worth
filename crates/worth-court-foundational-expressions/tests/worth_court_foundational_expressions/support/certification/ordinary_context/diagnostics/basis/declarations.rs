pub(crate) use {worth_foundational::compare_canonical_basis, worth_foundational::compare_diagnostic_explanation_bundles, worth_foundational::compare_diagnostic_support_reports, worth_foundational::foundational_diagnostic_canonical_basis_entries, worth_foundational::prepare_canonical_comparison, worth_foundational::prepare_diagnostic_explanation_bundle_for_canonical_basis, worth_foundational::prepare_diagnostic_support_report_for_canonical_basis, worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalComparisonOutcome, worth_foundational::CanonicalEquivalenceBasis, worth_foundational::FoundationalDiagnosticEvidencePosture};
pub(crate) use {worth_proof::TransitionOutcome};

pub(crate) use {crate::certification::ordinary_context::diagnostics::basis_support::diagnostic_bool_entry, crate::certification::ordinary_context::diagnostics::basis_support::diagnostic_integer_entry, crate::certification::ordinary_context::diagnostics::basis_support::diagnostic_text_entry, crate::certification::ordinary_context::diagnostics::basis_support::explanation_bundle_with_mixed_rows, crate::certification::ordinary_context::diagnostics::basis_support::explanation_bundle_with_tied_common_rows, crate::certification::ordinary_context::diagnostics::basis_support::support_report_equivalent_reordered, crate::certification::ordinary_context::diagnostics::basis_support::support_report_with_unsorted_inputs, crate::certification::ordinary_context::diagnostics::basis_support::version};

pub(crate) fn ready_support(
    report: &worth_foundational::FoundationalDiagnosticSupportReport,
) -> worth_foundational::CanonicalBasisReadyArtifact {
    match prepare_diagnostic_support_report_for_canonical_basis(version(), report) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready support basis"),
    }
}

pub(crate) fn ready_explanation(
    bundle: &worth_foundational::FoundationalDiagnosticExplanationBundle,
) -> worth_foundational::CanonicalBasisReadyArtifact {
    match prepare_diagnostic_explanation_bundle_for_canonical_basis(version(), bundle) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready explanation basis"),
    }
}

pub(crate) fn exact_compare(
    left: worth_foundational::CanonicalBasisReadyArtifact,
    right: worth_foundational::CanonicalBasisReadyArtifact,
) -> CanonicalComparisonOutcome {
    let ready = match prepare_canonical_comparison(
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left,
        right,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected comparison readiness"),
    };

    compare_canonical_basis(&ready)
}

pub(crate) fn assert_entries_present(entries: &[CanonicalBasisEntry], expected: &[CanonicalBasisEntry]) {
    for entry in expected {
        assert!(
            entries.contains(entry),
            "expected canonical entry missing: {entry:?}"
        );
    }
}