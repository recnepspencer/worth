#[allow(unused_imports)] use crate::certification::ordinary_context::diagnostics::basis::*;
use {worth_foundational::compare_canonical_basis, worth_foundational::compare_diagnostic_explanation_bundles, worth_foundational::compare_diagnostic_support_reports, worth_foundational::foundational_diagnostic_canonical_basis_entries, worth_foundational::prepare_canonical_comparison, worth_foundational::prepare_diagnostic_explanation_bundle_for_canonical_basis, worth_foundational::prepare_diagnostic_support_report_for_canonical_basis, worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalComparisonOutcome, worth_foundational::CanonicalEquivalenceBasis, worth_foundational::FoundationalDiagnosticEvidencePosture};
use {worth_proof::TransitionOutcome};

use {crate::certification::ordinary_context::diagnostics::basis_support::diagnostic_bool_entry, crate::certification::ordinary_context::diagnostics::basis_support::diagnostic_integer_entry, crate::certification::ordinary_context::diagnostics::basis_support::diagnostic_text_entry, crate::certification::ordinary_context::diagnostics::basis_support::explanation_bundle_with_mixed_rows, crate::certification::ordinary_context::diagnostics::basis_support::explanation_bundle_with_tied_common_rows, crate::certification::ordinary_context::diagnostics::basis_support::support_report_equivalent_reordered, crate::certification::ordinary_context::diagnostics::basis_support::support_report_with_unsorted_inputs, crate::certification::ordinary_context::diagnostics::basis_support::version};

#[test]
fn diagnostic_surfaces_canonicalize_the_same_across_independent_producers() {
    let left = support_report_with_unsorted_inputs();
    let right = support_report_equivalent_reordered();

    let left_ready = ready_support(&left);
    let right_ready = ready_support(&right);
    assert!(matches!(
        exact_compare(left_ready, right_ready),
        worth_foundational::CanonicalComparisonOutcome::Equivalent(_)
    ));
}

#[test]
fn diagnostic_row_canonicalization_does_not_depend_on_input_order_when_common_fields_tie() {
    let left = explanation_bundle_with_tied_common_rows(false);
    let right = explanation_bundle_with_tied_common_rows(true);

    let left_ready = ready_explanation(&left);
    let right_ready = ready_explanation(&right);
    assert!(matches!(
        exact_compare(left_ready, right_ready),
        worth_foundational::CanonicalComparisonOutcome::Equivalent(_)
    ));
}

#[test]
fn diagnostic_basis_preserves_bundle_row_gap_and_evidence_posture_meaning() {
    let bundle =
        explanation_bundle_with_mixed_rows(worth_foundational::FoundationalDiagnosticEvidencePosture::Summarized);
    let ready = ready_explanation(&bundle);
    let entries = foundational_diagnostic_canonical_basis_entries(&ready);

    assert_entries_present(
        entries,
        &[
            diagnostic_text_entry(
                worth_foundational::CanonicalBasisEntryKind::DiagnosticBundle,
                "bundle.artifact_kind",
                "explanation-bundle",
            ),
            diagnostic_text_entry(
                worth_foundational::CanonicalBasisEntryKind::DiagnosticBundle,
                "bundle.partiality",
                "partial-with-named-gaps",
            ),
            diagnostic_integer_entry(
                worth_foundational::CanonicalBasisEntryKind::DiagnosticBundle,
                "bundle.row_count",
                4,
            ),
            diagnostic_text_entry(
                worth_foundational::CanonicalBasisEntryKind::DiagnosticGap,
                "bundle.gap.0.class",
                "coverage-omission",
            ),
            diagnostic_text_entry(
                worth_foundational::CanonicalBasisEntryKind::DiagnosticGap,
                "bundle.gap.0.target_kind",
                "locator",
            ),
            diagnostic_text_entry(
                worth_foundational::CanonicalBasisEntryKind::DiagnosticRow,
                "bundle.row.0.family",
                "comparison",
            ),
            diagnostic_text_entry(
                worth_foundational::CanonicalBasisEntryKind::DiagnosticRow,
                "bundle.row.1.family",
                "decision",
            ),
            diagnostic_text_entry(
                worth_foundational::CanonicalBasisEntryKind::DiagnosticRow,
                "bundle.row.0.evidence_posture",
                "summarized",
            ),
            diagnostic_bool_entry(
                worth_foundational::CanonicalBasisEntryKind::DiagnosticRow,
                "bundle.row.0.has_mismatch_locator",
                true,
            ),
            diagnostic_text_entry(
                worth_foundational::CanonicalBasisEntryKind::DiagnosticRow,
                "bundle.row.2.evidence_origin_locator",
                "locator.boundary_artifact:51:proofs",
            ),
        ],
    );
}

#[test]
fn diagnostic_comparison_bundle_preserves_mismatch_basis_explicitly() {
    let left =
        explanation_bundle_with_mixed_rows(worth_foundational::FoundationalDiagnosticEvidencePosture::Summarized);
    let right =
        explanation_bundle_with_mixed_rows(worth_foundational::FoundationalDiagnosticEvidencePosture::RetainedDirect);

    let comparison = match compare_diagnostic_explanation_bundles(version(), &left, &right) {
        worth_proof::TransitionOutcome::Success(bundle) => bundle,
        outcome => panic!("expected diagnostic comparison bundle, got {outcome:?}"),
    };

    assert_eq!(comparison.left_row_count(), 4);
    assert_eq!(comparison.right_row_count(), 4);
    let mismatch = comparison
        .mismatch_basis()
        .expect("comparison should preserve mismatch basis");
    assert_eq!(
        mismatch.left_domain(),
        worth_foundational::CanonicalBasisDomain::Diagnostic
    );
    assert_eq!(
        mismatch.right_domain(),
        worth_foundational::CanonicalBasisDomain::Diagnostic
    );
    assert_eq!(
        mismatch.left_entry_kind(),
        Some(worth_foundational::CanonicalBasisEntryKind::DiagnosticRow)
    );
    assert_eq!(
        mismatch.right_entry_kind(),
        Some(worth_foundational::CanonicalBasisEntryKind::DiagnosticRow)
    );
}

#[test]
fn blind_consumers_can_read_family_distinct_rows_without_producer_state() {
    let bundle =
        explanation_bundle_with_mixed_rows(worth_foundational::FoundationalDiagnosticEvidencePosture::Summarized);

    let decision_codes = bundle
        .decision_rows()
        .map(|row| row.code().as_str())
        .collect::<Vec<_>>();
    let comparison_codes = bundle
        .comparison_rows()
        .map(|row| row.code().as_str())
        .collect::<Vec<_>>();
    let support_codes = bundle
        .support_rows()
        .map(|row| row.code().as_str())
        .collect::<Vec<_>>();
    let provenance_codes = bundle
        .provenance_ready_rows()
        .map(|row| row.code().as_str())
        .collect::<Vec<_>>();

    assert_eq!(decision_codes, vec!["decision.branch"]);
    assert_eq!(comparison_codes, vec!["comparison.parity"]);
    assert_eq!(support_codes, vec!["support.required"]);
    assert_eq!(provenance_codes, vec!["provenance.origin"]);

    let report = support_report_with_unsorted_inputs();
    let support_only = report
        .support_rows()
        .map(|row| row.code().as_str())
        .collect::<Vec<_>>();
    assert_eq!(support_only, vec!["support.required", "support.standard"]);
}

#[test]
fn support_report_comparison_bundle_reuses_same_mismatch_surface() {
    let left = support_report_with_unsorted_inputs();
    let right = support_report_equivalent_reordered();

    let comparison = match compare_diagnostic_support_reports(version(), &left, &right) {
        worth_proof::TransitionOutcome::Success(bundle) => bundle,
        outcome => panic!("expected support comparison bundle, got {outcome:?}"),
    };

    assert!(matches!(
        comparison.outcome(),
        worth_foundational::CanonicalComparisonOutcome::Equivalent(_)
    ));
    assert_eq!(
        comparison.left_artifact_kind(),
        worth_foundational::FoundationalDiagnosticArtifactKind::SupportReport
    );
}
