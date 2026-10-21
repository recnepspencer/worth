//! Availability is non-driving, but its source and image remain commit dependencies.
use super::*;
mod validation;
use crate::domain_computation::primary_graph::application_attempt::Movement;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as Fact;
use crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop as VerificationStop;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SourcePublicationReadFence;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphApplicationDecisionFact as DecisionFact;

pub(super) fn prepare(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
    before: &worth_relational::facade::snapshots::SnapshotHandle,
) -> Result<
    Option<SourcePublicationReadFence>,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let stop = consumed_output::verification_stop;
    let map_admission = crate::domain_computation::primary_graph::invariant_projection::consumed_output::map_admission_stop;
    let count = attempt.facts().len();

    let mut observations = attempt
        .facts()
        .values()
        .filter_map(|fact| match fact {
            DecisionFact::Application { fact, .. }
            | DecisionFact::ObservedSource { fact }
            | DecisionFact::ApplicationObservedSource { fact, .. } => match fact.as_ref() {
                Fact::SourcePublication { observation } => Some(std::sync::Arc::clone(observation)),
                _ => None,
            },
            _ => None,
        })
        .peekable();
    if observations.peek().is_none() {
        return Ok(None);
    }

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
            .source_publication_read_fence(&retained, &selected)
            .map_err(|cause| stop(map_admission(cause)))?;
        Ok::<_, crate::domain_computation::WorthQueryProviderSessionCommitStop>(fence)
    })?;
    validation::validate(execution, provider, before, &retained)?;
    Ok(Some(fence))
}
