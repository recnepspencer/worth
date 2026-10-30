pub use worth_proof::TransitionOutcome;
pub use {
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

pub use crate::certification::ordinary_context::transitions::fixtures::branch::branch_id;
pub use {
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::no_op_for_aspect,
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_aspect,
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_node,
};

pub fn aspect_scope() -> FoundationalMergeScope {
    worth_foundational::FoundationalMergeScope::selected_aspects([
        selected_aspect("gear", "teeth"),
        selected_aspect("gear", "thickness"),
    ])
    .expect("aspect scope")
}

pub fn materialize(
    input: worth_foundational::FoundationalDiagnosticExplanationInput,
    richness: DiagnosticRichnessProfile,
) -> worth_foundational::FoundationalDiagnosticExplanationBundle {
    materialize_diagnostic_explanation_bundle(
        input,
        profile(richness),
        worth_foundational::FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("diagnostic explanation")
}

pub fn profile(richness: DiagnosticRichnessProfile) -> FoundationalProfileSet {
    worth_foundational::FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: richness,
        support_posture: worth_foundational::SupportPostureProfile::SupportReady,
        compatibility_posture: worth_foundational::CompatibilityPostureProfile::NativeOnly,
        admission_readiness: worth_foundational::AdmissionReadinessProfile::Admitted,
        retention_delivery: worth_foundational::RetentionDeliveryProfile::Retained,
        certification_posture: worth_foundational::CertificationPostureProfile::Uncertified,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    })
    .expect("profile")
}

pub fn row_codes(rows: &[worth_foundational::FoundationalDiagnosticRow]) -> Vec<&str> {
    rows.iter().map(|row| row.code().as_str()).collect()
}

pub fn locator_entries(locator: FoundationalTransitionLocator) -> Vec<CanonicalBasisEntry> {
    ready_locator(locator).payload().entries().to_vec()
}

pub fn ready_locator(locator: FoundationalTransitionLocator) -> CanonicalBasisReadyArtifact {
    match prepare_locator_for_canonical_basis(
        version(),
        worth_foundational::CanonicalLocatorInput::Transition(locator),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready transition locator"),
    }
}

pub fn exact_compare(
    left: CanonicalBasisReadyArtifact,
    right: CanonicalBasisReadyArtifact,
) -> CanonicalComparisonOutcome {
    let ready = match prepare_canonical_comparison(
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left,
        right,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected comparison readiness"),
    };
    worth_foundational::compare_canonical_basis(&ready)
}

pub fn transition_locator_text_entry(locus: &str, value: &str) -> CanonicalBasisEntry {
    worth_foundational::CanonicalBasisEntry::new(
        worth_foundational::CanonicalBasisDomain::Locator,
        worth_foundational::CanonicalBasisLocus::Named(locus.into()),
        worth_foundational::CanonicalBasisEntryKind::TransitionLocator,
        worth_foundational::CanonicalBasisValue::ExactText(value.into()),
    )
}

pub fn version() -> CanonicalizationRuleVersion {
    worth_foundational::CanonicalizationRuleVersion::new("milestone-9-phase-13").expect("version")
}
