//! Bind retained whole-Source expectations after actual idempotency resolution.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact as Fact,
    provider::WorthQueryPrimaryGraphApplicationDecisionFact as DecisionFact,
};

pub(super) fn bind(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
    candidate: &mut worth_relational::facade::mvcc::PreparedRelationalCommitCandidate,
) -> Result<(), crate::domain_computation::WorthQueryProviderSessionCommitStop> {
    for fact in attempt.facts().values() {
        let (DecisionFact::Application { fact, .. }
        | DecisionFact::ObservedSource { fact }
        | DecisionFact::ApplicationObservedSource { fact, .. }) = fact
        else {
            continue;
        };
        if let Fact::ManagedIndexSource { observation } = fact.as_ref() {
            // This receiver reborrows the genuine current request and pins the
            // actual selected cell image through fixed compound preflight.
            candidate
                .require_managed_source(execution, observation.native())
                .map_err(native_output_witness_stop)?;
        }
    }
    Ok(())
}
