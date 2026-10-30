#[allow(unused_imports)]
use crate::certification::ordinary_context::transitions::scoped_merge_diagnostics::*;
use worth_proof::TransitionOutcome;
use {
    worth_foundational::materialize_diagnostic_explanation_bundle,
    worth_foundational::prepare_canonical_comparison,
    worth_foundational::prepare_locator_for_canonical_basis,
    worth_foundational::prepare_scoped_merge_diagnostic_explanation,
    worth_foundational::AdmissionReadinessProfile, worth_foundational::CanonicalBasisDomain,
    worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind,
    worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisReadyArtifact,
    worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalComparisonOutcome,
    worth_foundational::CanonicalEquivalenceBasis, worth_foundational::CanonicalLocatorInput,
    worth_foundational::CanonicalizationRuleVersion,
    worth_foundational::CertificationPostureProfile,
    worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile,
    worth_foundational::FoundationalAdmittedMergeScopeEvidence,
    worth_foundational::FoundationalDiagnosticDeliveryClass,
    worth_foundational::FoundationalDiagnosticLocator, worth_foundational::FoundationalMergeScope,
    worth_foundational::FoundationalMergeScopeFamily,
    worth_foundational::FoundationalMergeScopeLocator, worth_foundational::FoundationalProfileSet,
    worth_foundational::FoundationalProfileSetInput,
    worth_foundational::FoundationalScopeAdmissionBasis,
    worth_foundational::FoundationalScopedMergeDenialEvidence,
    worth_foundational::FoundationalScopedMergeDenialKind,
    worth_foundational::FoundationalScopedMergeDiagnosticInput,
    worth_foundational::FoundationalScopedMergeUnavailablePosture,
    worth_foundational::FoundationalScopedMergeUnavailableReason,
    worth_foundational::FoundationalSelectedAspectScopeLocator,
    worth_foundational::FoundationalSelectedScopeNoOpCause,
    worth_foundational::FoundationalSkippedOutOfScopeEvidence,
    worth_foundational::FoundationalTransitionLocator,
    worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile,
};

use crate::certification::ordinary_context::transitions::fixtures::branch::branch_id;
use {
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::no_op_for_aspect,
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_aspect,
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_node,
};

#[test]
fn scope_request_explanation_keeps_compact_request_and_elides_loci_when_minimal() {
    let scope = aspect_scope();
    let input = prepare_scoped_merge_diagnostic_explanation(
        worth_foundational::FoundationalScopedMergeDiagnosticInput::ScopeRequest {
            source_branch: branch_id("feature/gear"),
            target_branch: branch_id("main"),
            requested_scope: scope,
        },
    );

    let minimal = materialize(
        input,
        worth_foundational::DiagnosticRichnessProfile::OperationalMinimal,
    );

    assert_eq!(row_codes(minimal.rows()), vec!["merge-scope.requested"]);
    assert_eq!(minimal.counter_snapshot().retained_evidence_count(), 2);
}

#[test]
fn admitted_scope_explanation_names_skipped_no_op_and_locus_details_by_profile() {
    let evidence = worth_foundational::FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/gear"),
        branch_id("main"),
        aspect_scope(),
        worth_foundational::FoundationalScopeAdmissionBasis::IdentityCorresponded,
        [],
        [selected_aspect("gear", "teeth")],
        [no_op_for_aspect(
            "gear",
            "thickness",
            worth_foundational::FoundationalSelectedScopeNoOpCause::EquivalentTargetTruth,
        )],
        worth_foundational::FoundationalSkippedOutOfScopeEvidence::new(3, None),
        2,
    )
    .expect("admitted scoped evidence");

    let input = prepare_scoped_merge_diagnostic_explanation(
        worth_foundational::FoundationalScopedMergeDiagnosticInput::AdmittedScope(evidence),
    );
    let standard = materialize(
        input.clone(),
        worth_foundational::DiagnosticRichnessProfile::Standard,
    );
    let forensic = materialize(
        input,
        worth_foundational::DiagnosticRichnessProfile::Forensic,
    );

    assert!(row_codes(standard.rows()).contains(&"merge-scope.admitted"));
    assert!(row_codes(standard.rows()).contains(&"merge-scope.skipped"));
    assert!(row_codes(standard.rows()).contains(&"merge-scope.no-op"));
    assert!(row_codes(standard.rows()).contains(&"merge-scope.admitted-locus"));
    assert!(!row_codes(standard.rows()).contains(&"merge-scope.no-op-locus"));
    assert!(row_codes(forensic.rows()).contains(&"merge-scope.no-op-locus"));
    assert_eq!(forensic.counter_snapshot().retained_evidence_count(), 3);
    assert_eq!(
        forensic.counter_snapshot().reconstructable_evidence_count(),
        4
    );
}

#[test]
fn denial_and_unavailable_explanations_attach_provenance_ready_scope_rows() {
    let denied = worth_foundational::FoundationalScopedMergeDenialEvidence::new(
        branch_id("feature/gear"),
        branch_id("main"),
        worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("gear")])
            .expect("scope"),
        worth_foundational::FoundationalScopedMergeDenialKind::UnknownSelectedNode,
        worth_foundational::FoundationalDeniedScopeLocus::Node(selected_node("gear")),
    )
    .expect("denial");
    let unavailable = worth_foundational::FoundationalScopedMergeUnavailablePosture::new(
        branch_id("feature/gear"),
        branch_id("main"),
        worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("scope"),
        worth_foundational::FoundationalScopedMergeUnavailableReason::RuntimeDoesNotSupportSelectedNodes,
    )
    .expect("unavailable");

    let denied_bundle = materialize(
        prepare_scoped_merge_diagnostic_explanation(
            worth_foundational::FoundationalScopedMergeDiagnosticInput::DeniedScope(denied),
        ),
        worth_foundational::DiagnosticRichnessProfile::Forensic,
    );
    let unavailable_bundle = materialize(
        prepare_scoped_merge_diagnostic_explanation(
            worth_foundational::FoundationalScopedMergeDiagnosticInput::UnavailableScope(
                unavailable,
            ),
        ),
        worth_foundational::DiagnosticRichnessProfile::Forensic,
    );

    assert_eq!(
        denied_bundle.outcome_kind(),
        worth_foundational::FoundationalDiagnosticOutcomeKind::Denied
    );
    assert!(row_codes(denied_bundle.rows()).contains(&"merge-scope.denied-origin"));
    assert_eq!(
        unavailable_bundle.outcome_kind(),
        worth_foundational::FoundationalDiagnosticOutcomeKind::Deferred
    );
    assert!(row_codes(unavailable_bundle.rows()).contains(&"merge-scope.unavailable-origin"));
}

#[test]
fn merge_scope_locator_has_canonical_basis_and_safe_diagnostic_fragments() {
    let entries = locator_entries(
        worth_foundational::FoundationalTransitionLocator::MergeScope(
            worth_foundational::FoundationalMergeScopeLocator::new(
                branch_id("feature/gear"),
                branch_id("main"),
                worth_foundational::FoundationalMergeScopeFamily::SelectedAspects,
            ),
        ),
    );

    assert_eq!(
        entries,
        vec![
            transition_locator_text_entry("transition.merge_scope.family", "selected-aspects"),
            transition_locator_text_entry("transition.merge_scope.kind", "merge-scope"),
            transition_locator_text_entry("transition.merge_scope.source_branch", "feature/gear"),
            transition_locator_text_entry("transition.merge_scope.target_branch", "main"),
        ]
    );

    let left = worth_foundational::FoundationalDiagnosticLocator::Transition(
        worth_foundational::FoundationalTransitionLocator::MergeScope(
            worth_foundational::FoundationalMergeScopeLocator::new(
                branch_id("feature:gear"),
                branch_id("main"),
                worth_foundational::FoundationalMergeScopeFamily::SelectedAspects,
            ),
        ),
    );
    let right = worth_foundational::FoundationalDiagnosticLocator::Transition(
        worth_foundational::FoundationalTransitionLocator::MergeScope(
            worth_foundational::FoundationalMergeScopeLocator::new(
                branch_id("feature"),
                branch_id("gear:main"),
                worth_foundational::FoundationalMergeScopeFamily::SelectedAspects,
            ),
        ),
    );
    assert_ne!(
        left.canonical_key_fragment(),
        right.canonical_key_fragment()
    );
}

#[test]
fn selected_locus_diagnostics_remain_boundary_attachment_compatible() {
    let selected_locus = worth_foundational::FoundationalTransitionLocator::SelectedAspectScope(
        worth_foundational::FoundationalSelectedAspectScopeLocator::new(
            branch_id("feature/gear"),
            branch_id("main"),
            selected_aspect("gear", "teeth"),
        ),
    );
    let scope_request = worth_foundational::FoundationalTransitionLocator::MergeScope(
        worth_foundational::FoundationalMergeScopeLocator::new(
            branch_id("feature/gear"),
            branch_id("main"),
            worth_foundational::FoundationalMergeScopeFamily::SelectedAspects,
        ),
    );

    let selected_ready = ready_locator(selected_locus);
    let scope_ready = ready_locator(scope_request);
    assert!(matches!(
        exact_compare(selected_ready, scope_ready),
        worth_foundational::CanonicalComparisonOutcome::Mismatched(_)
    ));
}
