use super::WorthUiApplicationSessionState;
use crate::runtime::observation::{
    lower_authored_differences, UiAuthoredSourceClassification, UiAuthoredSourceSuccession,
    UiChangeClassificationDenial, UiChangeClassificationOutcome, UiChangeClassificationRequest,
    UiChangeClassifier,
};
use crate::runtime::{WorthUiCandidateAdmission, WorthUiRuntimeArtifactComparisonOutcome};

struct PreparedAuthoredSource {
    successor: crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    candidate: crate::runtime::WorthUiReplacementCandidate,
    evidence_changed: bool,
}

struct AdmittedAuthoredSource {
    successor: crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    admitted: crate::runtime::WorthUiAdmittedReplacementCandidate,
    evidence_changed: bool,
}

struct ComparedAuthoredSource {
    successor: crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    admitted: crate::runtime::WorthUiAdmittedReplacementCandidate,
    comparison: crate::runtime::WorthUiRuntimeArtifactComparison,
    evidence_changed: bool,
}

impl WorthUiApplicationSessionState {
    pub(crate) fn validate_observation_basis(
        &self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        set: &crate::runtime::observation::UiAdmittedObservationSet,
    ) -> Result<(), UiChangeClassificationDenial> {
        UiChangeClassifier::validate_basis(set, session, self.app.capabilities().digest().as_u64())
    }

    pub(crate) fn intent_consequence_fact_capacity(&self) -> usize {
        self.app
            .prepared_authority()
            .change_profile()
            .rebind()
            .budget()
            .changed_facts
    }

    pub(crate) fn classify_intent_consequence(
        &self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        set: crate::runtime::observation::UiAdmittedObservationSet,
    ) -> crate::runtime::observation::UiClassifiedChange {
        UiChangeClassifier::classify_intent_consequence(
            set,
            session,
            self.app.capabilities().digest().as_u64(),
            self.app.generation_identity().clone(),
            self.intent_consequence_fact_capacity(),
        )
    }

    pub(crate) fn classify_observations(
        &self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        set: crate::runtime::observation::UiAdmittedObservationSet,
    ) -> Result<UiChangeClassificationOutcome, UiChangeClassificationDenial> {
        let source_basis = self.app.capabilities().digest().as_u64();
        let budget = self
            .app
            .prepared_authority()
            .change_profile()
            .rebind()
            .budget();
        UiChangeClassifier::classify(UiChangeClassificationRequest {
            set,
            expected_session: session,
            expected_source_basis: source_basis,
            predecessor_generation: self.app.generation_identity().clone(),
            fact_limit: budget.changed_facts,
            classify_source: |submission| {
                self.classify_authored_source(
                    submission,
                    budget.changed_facts,
                    budget.comparison_structural_entries,
                )
            },
        })
    }

    pub(crate) fn resolve_affected_scope(
        &self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        change: crate::runtime::observation::UiClassifiedChange,
    ) -> Result<
        crate::runtime::rebind::UiResolvedAffectedScope,
        crate::runtime::rebind::UiAffectedScopeDenial,
    > {
        crate::runtime::rebind::UiAffectedScopeResolver::resolve(
            change,
            session,
            self.app.prepared_authority(),
        )
    }

    fn classify_authored_source(
        &self,
        submission: crate::runtime::WorthUiWatchedCandidateSubmission,
        fact_limit: usize,
        structural_entry_limit: usize,
    ) -> Result<UiAuthoredSourceClassification, UiChangeClassificationDenial> {
        let prepared = prepare_authored_source(self.app.prepared_authority(), submission)?;
        let admitted = admit_authored_source(&self.runtime, prepared)?;
        let compared = compare_authored_source(&self.runtime, admitted, structural_entry_limit)?;
        finish_authored_classification(
            self.app.prepared_authority(),
            &self.runtime,
            compared,
            fact_limit,
        )
    }
}

#[inline(never)]
fn prepare_authored_source(
    current: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    submission: crate::runtime::WorthUiWatchedCandidateSubmission,
) -> Result<PreparedAuthoredSource, UiChangeClassificationDenial> {
    let (successor, candidate) =
        crate::facade::lifecycle::prepare_successor_application_authority(current, submission)
            .map_err(|denial| UiChangeClassificationDenial::SourcePreparation(Box::new(denial)))?;
    let evidence_changed = current.authored_source_basis() != successor.authored_source_basis();
    Ok(PreparedAuthoredSource {
        successor,
        candidate,
        evidence_changed,
    })
}

#[inline(never)]
fn admit_authored_source(
    runtime: &crate::runtime::WorthUiRuntime,
    prepared: PreparedAuthoredSource,
) -> Result<AdmittedAuthoredSource, UiChangeClassificationDenial> {
    let admitted =
        WorthUiCandidateAdmission::for_active_basis(runtime.replacement_admission_basis())
            .admit(prepared.candidate)
            .map_err(|report| UiChangeClassificationDenial::CandidateAdmission(Box::new(report)))?;
    Ok(AdmittedAuthoredSource {
        successor: prepared.successor,
        admitted,
        evidence_changed: prepared.evidence_changed,
    })
}

#[inline(never)]
fn compare_authored_source(
    runtime: &crate::runtime::WorthUiRuntime,
    admitted: AdmittedAuthoredSource,
    structural_entry_limit: usize,
) -> Result<ComparedAuthoredSource, UiChangeClassificationDenial> {
    let comparison = runtime
        .compare_admitted_replacement_bounded(&admitted.admitted, structural_entry_limit)
        .map_err(UiChangeClassificationDenial::ArtifactComparison)?;
    Ok(ComparedAuthoredSource {
        successor: admitted.successor,
        admitted: admitted.admitted,
        comparison,
        evidence_changed: admitted.evidence_changed,
    })
}

#[inline(never)]
fn finish_authored_classification(
    current: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    runtime: &crate::runtime::WorthUiRuntime,
    compared: ComparedAuthoredSource,
    fact_limit: usize,
) -> Result<UiAuthoredSourceClassification, UiChangeClassificationDenial> {
    let ComparedAuthoredSource {
        successor,
        admitted,
        comparison,
        evidence_changed,
    } = compared;
    let facts = lower_authored_differences(&comparison, current, &successor, fact_limit)?;
    let meaning_changed = comparison.outcome()
        == WorthUiRuntimeArtifactComparisonOutcome::MeaningfullyDifferent
        || !facts.is_empty();
    match (meaning_changed, evidence_changed) {
        (false, false) => Ok(UiAuthoredSourceClassification::ObservedNoChange),
        (false, true) => Ok(UiAuthoredSourceClassification::EvidenceOnly(
            UiAuthoredSourceSuccession::EvidenceOnly {
                successor_authority: successor,
                admitted_candidate: admitted,
                comparison,
            },
        )),
        (true, _) => {
            let replacement =
                plan_changed_source_replacement(runtime, admitted, &comparison, &successor)?;
            let identity_lifecycle_index =
                crate::runtime::rebind::UiSourceIdentityLifecycleIndex::build(
                    current,
                    &successor,
                    &replacement.node_plan,
                )
                .map_err(|denial| {
                    UiChangeClassificationDenial::IdentityLifecycle(Box::new(denial))
                })?;
            Ok(UiAuthoredSourceClassification::Changed {
                facts,
                succession: UiAuthoredSourceSuccession::Changed {
                    successor_authority: successor,
                    comparison,
                    replacement,
                    identity_lifecycle_index,
                },
            })
        }
    }
}

#[inline(never)]
fn plan_changed_source_replacement(
    runtime: &crate::runtime::WorthUiRuntime,
    admitted: crate::runtime::WorthUiAdmittedReplacementCandidate,
    comparison: &crate::runtime::WorthUiRuntimeArtifactComparison,
    successor: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
) -> Result<
    crate::runtime::replacement::WorthUiReplacementNodePlanReady,
    UiChangeClassificationDenial,
> {
    let candidate_query_binding = successor.query_binding_plan().prepare_downstream_state();
    runtime
        .prepare_replacement_node_plan_from_comparison(
            admitted,
            comparison.clone(),
            successor.query_binding_plan(),
            &candidate_query_binding,
        )
        .map_err(|denial| UiChangeClassificationDenial::ReplacementPlanning(Box::new(denial)))
}
