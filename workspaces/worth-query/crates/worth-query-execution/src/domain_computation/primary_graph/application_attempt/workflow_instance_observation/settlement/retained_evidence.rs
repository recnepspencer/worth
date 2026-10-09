use crate::domain_computation::primary_graph::application_attempt::check_request_live;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::{
    instance::WorkflowAssessmentEvidenceLocator, schema::WorthQueryWorkflowLayout,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

use super::super::ObservedWorkflowAssessmentEvidence;

pub(in crate::domain_computation::primary_graph::application_attempt) fn observe_retained_assessment_evidence(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    locator: WorkflowAssessmentEvidenceLocator,
    observation_request: &WorthQueryRequestScope,
) -> Result<
    (
        ObservedWorkflowAssessmentEvidence,
        Vec<WorthQueryApplicationObservedFact>,
    ),
    WorthQueryApplicationAttemptDenial,
> {
    let transition = locator.transition();
    let mut facts = Vec::new();
    super::observe_retained_transition(runtime, snapshot, layout, transition, &mut facts)?;
    let evidence = super::observe_assessment_evidence(
        runtime,
        snapshot,
        layout,
        transition.entity(),
        &mut facts,
    )?
    .filter(|evidence| evidence.entity == locator.evidence())
    .ok_or_else(|| mismatch("retained assessment evidence locator changed"))?;
    check_request_live(observation_request, "workflow retained evidence")?;
    Ok((evidence, facts))
}

fn mismatch(subject: &'static str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}
