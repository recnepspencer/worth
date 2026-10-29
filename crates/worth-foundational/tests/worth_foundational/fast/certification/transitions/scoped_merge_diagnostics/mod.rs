use worth_foundational::{
    materialize_diagnostic_explanation_bundle, prepare_canonical_comparison,
    prepare_locator_for_canonical_basis, prepare_scoped_merge_diagnostic_explanation,
    AdmissionReadinessProfile, CanonicalBasisDomain, CanonicalBasisEntry, CanonicalBasisEntryKind,
    CanonicalBasisLocus, CanonicalBasisReadyArtifact, CanonicalBasisValue,
    CanonicalComparisonOutcome, CanonicalEquivalenceBasis, CanonicalLocatorInput,
    CanonicalizationRuleVersion, CertificationPostureProfile, CompatibilityPostureProfile,
    DiagnosticRichnessProfile, FoundationalAdmittedMergeScopeEvidence,
    FoundationalDiagnosticDeliveryClass, FoundationalDiagnosticLocator, FoundationalMergeScope,
    FoundationalMergeScopeFamily, FoundationalMergeScopeLocator, FoundationalProfileSet,
    FoundationalProfileSetInput, FoundationalScopeAdmissionBasis,
    FoundationalScopedMergeDenialEvidence, FoundationalScopedMergeDenialKind,
    FoundationalScopedMergeDiagnosticInput, FoundationalScopedMergeUnavailablePosture,
    FoundationalScopedMergeUnavailableReason, FoundationalSelectedAspectScopeLocator,
    FoundationalSelectedScopeNoOpCause, FoundationalSkippedOutOfScopeEvidence,
    FoundationalTransitionLocator, RetentionDeliveryProfile, SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

use super::fixtures::branch::branch_id;
use super::fixtures::scoped_merge::{no_op_for_aspect, selected_aspect, selected_node};

#[test]
fn scope_request_explanation_keeps_compact_request_and_elides_loci_when_minimal() {
    let scope = aspect_scope();
    let input = prepare_scoped_merge_diagnostic_explanation(
        FoundationalScopedMergeDiagnosticInput::ScopeRequest {
            source_branch: branch_id("feature/gear"),
            target_branch: branch_id("main"),
            requested_scope: scope,
        },
    );

    let minimal = materialize(input, DiagnosticRichnessProfile::OperationalMinimal);

    assert_eq!(row_codes(minimal.rows()), vec!["merge-scope.requested"]);
    assert_eq!(minimal.counter_snapshot().retained_evidence_count(), 2);
}

#[test]
fn admitted_scope_explanation_names_skipped_no_op_and_locus_details_by_profile() {
    let evidence = FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/gear"),
        branch_id("main"),
        aspect_scope(),
        FoundationalScopeAdmissionBasis::IdentityCorresponded,
        [],
        [selected_aspect("gear", "teeth")],
        [no_op_for_aspect(
            "gear",
            "thickness",
            FoundationalSelectedScopeNoOpCause::EquivalentTargetTruth,
        )],
        FoundationalSkippedOutOfScopeEvidence::new(3, None),
        2,
    )
    .expect("admitted scoped evidence");

    let input = prepare_scoped_merge_diagnostic_explanation(
        FoundationalScopedMergeDiagnosticInput::AdmittedScope(evidence),
    );
    let standard = materialize(input.clone(), DiagnosticRichnessProfile::Standard);
    let forensic = materialize(input, DiagnosticRichnessProfile::Forensic);

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
    let denied = FoundationalScopedMergeDenialEvidence::new(
        branch_id("feature/gear"),
        branch_id("main"),
        FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("scope"),
        FoundationalScopedMergeDenialKind::UnknownSelectedNode,
        worth_foundational::FoundationalDeniedScopeLocus::Node(selected_node("gear")),
    )
    .expect("denial");
    let unavailable = FoundationalScopedMergeUnavailablePosture::new(
        branch_id("feature/gear"),
        branch_id("main"),
        FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("scope"),
        FoundationalScopedMergeUnavailableReason::RuntimeDoesNotSupportSelectedNodes,
    )
    .expect("unavailable");

    let denied_bundle = materialize(
        prepare_scoped_merge_diagnostic_explanation(
            FoundationalScopedMergeDiagnosticInput::DeniedScope(denied),
        ),
        DiagnosticRichnessProfile::Forensic,
    );
    let unavailable_bundle = materialize(
        prepare_scoped_merge_diagnostic_explanation(
            FoundationalScopedMergeDiagnosticInput::UnavailableScope(unavailable),
        ),
        DiagnosticRichnessProfile::Forensic,
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
    let entries = locator_entries(FoundationalTransitionLocator::MergeScope(
        FoundationalMergeScopeLocator::new(
            branch_id("feature/gear"),
            branch_id("main"),
            FoundationalMergeScopeFamily::SelectedAspects,
        ),
    ));

    assert_eq!(
        entries,
        vec![
            transition_locator_text_entry("transition.merge_scope.family", "selected-aspects"),
            transition_locator_text_entry("transition.merge_scope.kind", "merge-scope"),
            transition_locator_text_entry("transition.merge_scope.source_branch", "feature/gear"),
            transition_locator_text_entry("transition.merge_scope.target_branch", "main"),
        ]
    );

    let left = FoundationalDiagnosticLocator::Transition(
        FoundationalTransitionLocator::MergeScope(FoundationalMergeScopeLocator::new(
            branch_id("feature:gear"),
            branch_id("main"),
            FoundationalMergeScopeFamily::SelectedAspects,
        )),
    );
    let right = FoundationalDiagnosticLocator::Transition(
        FoundationalTransitionLocator::MergeScope(FoundationalMergeScopeLocator::new(
            branch_id("feature"),
            branch_id("gear:main"),
            FoundationalMergeScopeFamily::SelectedAspects,
        )),
    );
    assert_ne!(
        left.canonical_key_fragment(),
        right.canonical_key_fragment()
    );
}

#[test]
fn selected_locus_diagnostics_remain_boundary_attachment_compatible() {
    let selected_locus = FoundationalTransitionLocator::SelectedAspectScope(
        FoundationalSelectedAspectScopeLocator::new(
            branch_id("feature/gear"),
            branch_id("main"),
            selected_aspect("gear", "teeth"),
        ),
    );
    let scope_request =
        FoundationalTransitionLocator::MergeScope(FoundationalMergeScopeLocator::new(
            branch_id("feature/gear"),
            branch_id("main"),
            FoundationalMergeScopeFamily::SelectedAspects,
        ));

    let selected_ready = ready_locator(selected_locus);
    let scope_ready = ready_locator(scope_request);
    assert!(matches!(
        exact_compare(selected_ready, scope_ready),
        CanonicalComparisonOutcome::Mismatched(_)
    ));
}

fn aspect_scope() -> FoundationalMergeScope {
    FoundationalMergeScope::selected_aspects([
        selected_aspect("gear", "teeth"),
        selected_aspect("gear", "thickness"),
    ])
    .expect("aspect scope")
}

fn materialize(
    input: worth_foundational::FoundationalDiagnosticExplanationInput,
    richness: DiagnosticRichnessProfile,
) -> worth_foundational::FoundationalDiagnosticExplanationBundle {
    materialize_diagnostic_explanation_bundle(
        input,
        profile(richness),
        FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("diagnostic explanation")
}

fn profile(richness: DiagnosticRichnessProfile) -> FoundationalProfileSet {
    FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: richness,
        support_posture: SupportPostureProfile::SupportReady,
        compatibility_posture: CompatibilityPostureProfile::NativeOnly,
        admission_readiness: AdmissionReadinessProfile::Admitted,
        retention_delivery: RetentionDeliveryProfile::Retained,
        certification_posture: CertificationPostureProfile::Uncertified,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    })
    .expect("profile")
}

fn row_codes(rows: &[worth_foundational::FoundationalDiagnosticRow]) -> Vec<&str> {
    rows.iter().map(|row| row.code().as_str()).collect()
}

fn locator_entries(locator: FoundationalTransitionLocator) -> Vec<CanonicalBasisEntry> {
    ready_locator(locator).payload().entries().to_vec()
}

fn ready_locator(locator: FoundationalTransitionLocator) -> CanonicalBasisReadyArtifact {
    match prepare_locator_for_canonical_basis(version(), CanonicalLocatorInput::Transition(locator))
    {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready transition locator"),
    }
}

fn exact_compare(
    left: CanonicalBasisReadyArtifact,
    right: CanonicalBasisReadyArtifact,
) -> CanonicalComparisonOutcome {
    let ready = match prepare_canonical_comparison(
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left,
        right,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected comparison readiness"),
    };
    worth_foundational::compare_canonical_basis(&ready)
}

fn transition_locator_text_entry(locus: &str, value: &str) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Locator,
        CanonicalBasisLocus::Named(locus.into()),
        CanonicalBasisEntryKind::TransitionLocator,
        CanonicalBasisValue::ExactText(value.into()),
    )
}

fn version() -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new("milestone-9-phase-13").expect("version")
}
