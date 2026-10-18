//! Availability is non-driving, but its source and image remain commit dependencies.
use super::*;
mod validation;
use crate::domain_computation::primary_graph::{
    application_attempt::{
        Movement, WorthQueryApplicationObservedFact as Fact, WorthQuerySourceCurrentnessFailure,
    },
    invariant_projection::ConsumedOutputVerificationStop as VerificationStop,
    output_lineage::invalidation::{InvalidationEditAdmission, SourcePublicationReadFence},
    provider::WorthQueryPrimaryGraphApplicationDecisionFact as DecisionFact,
};

pub(super) fn prepare(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
    before: &worth_relational::facade::snapshots::SnapshotHandle,
    admission: &mut InvalidationEditAdmission,
) -> Result<
    Option<SourcePublicationReadFence>,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let stop = consumed_output::verification_stop;
    let map_admission = crate::domain_computation::primary_graph::invariant_projection::consumed_output::map_admission_stop;
    let count = attempt.facts().len();
    admission
        .charge_external_work(count as u64)
        .map_err(|cause| stop(map_admission(cause)))?;
    let observations = attempt.facts().values().filter_map(|fact| match fact {
        DecisionFact::Application { fact, .. }
        | DecisionFact::ObservedSource { fact }
        | DecisionFact::ApplicationObservedSource { fact, .. } => match fact.as_ref() {
            Fact::SourcePublication { observation } => Some(observation),
            _ => None,
        },
        _ => None,
    });
    if observations.clone().next().is_none() {
        return Ok(None);
    }
    let bytes = count.checked_mul(std::mem::size_of::<&crate::domain_computation::primary_graph::output_lineage::invalidation::ObservedSourcePublication>())
        .and_then(|bytes| u64::try_from(bytes).ok()).ok_or_else(|| stop(VerificationStop::WorkExhausted))?;
    admission
        .admit_read_scratch(bytes)
        .map_err(|cause| stop(map_admission(cause)))?;
    let mut retained = Vec::with_capacity(count);
    retained.extend(observations);
    let fence = provider.graph.with_runtime(execution, |runtime| {
        // Select the original image before validating any nested availability.
        // Even a completion that leaves every outer slot unchanged must fail
        // the SAME native image condition at preflight after this validation.
        let selected = runtime
            .read_truth()
            .positioned_snapshot(before)
            .map_err(|_| stop(VerificationStop::Unavailable))?;
        let fence = provider
            .graph
            .source_owner
            .invalidation_owner
            .source_publication_read_fence(&retained, &selected, admission)
            .map_err(|cause| stop(map_admission(cause)))?;
        Ok::<_, crate::domain_computation::WorthQueryProviderSessionCommitStop>(fence)
    })?;
    validation::validate(execution, provider, before, &retained, admission)?;
    Ok(Some(fence))
}
