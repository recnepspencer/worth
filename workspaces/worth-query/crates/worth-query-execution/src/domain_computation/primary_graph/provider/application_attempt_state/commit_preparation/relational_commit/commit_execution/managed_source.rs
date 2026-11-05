//! Bind the checked managed Source target after actual idempotency resolution.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact as Fact,
    provider::WorthQueryPrimaryGraphApplicationDecisionFact as DecisionFact,
};

pub(super) fn bind(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
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
            let checked = provider.graph.with_runtime(execution, |runtime| observation.checked_at(runtime, execution, snapshot))
                .map_err(comparison_stop)?
                .ok_or_else(|| native_output_witness_stop(worth_relational::facade::mvcc::CompanionPreflightStop::ManagedSourceChanged))?;
            if checked.is_realized() {
                provider
                    .application_attempt_operation_control
                    .after_managed_source_revalidation();
            }
            // This receiver reborrows the genuine current request and pins the
            // checked target cell image through fixed compound preflight. The
            // runtime loan above is released before this final pin operation.
            candidate
                .require_managed_source(execution, checked.native())
                .map_err(native_output_witness_stop)?;
        }
    }
    Ok(())
}

fn comparison_stop(
    cause: crate::domain_computation::primary_graph::vocabulary::application_attempt::source_currentness::WorthQuerySourceCurrentnessFailure,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    use crate::domain_computation::primary_graph::vocabulary::application_attempt::source_currentness::WorthQuerySourceCurrentnessFailure as Stop;
    match cause {
        Stop::ManagedIndex(cause) => crate::domain_computation::WorthQueryProviderSessionCommitStop::PreEffectDenied(
            provider_failure(WorthQueryProviderSessionProtocolStage::Commit, "managed Source consistency refused")
                .with_managed_index_denial(cause)),
        Stop::Execution(cause) => crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_stop(cause, None),
        Stop::OutputObservation(cause) => super::consumed_output::verification_stop(cause),
        Stop::WorkBudgetExceeded => super::consumed_output::verification_stop(
            crate::domain_computation::primary_graph::vocabulary::invariant_projection::verification::ConsumedOutputVerificationStop::WorkExhausted),
        Stop::Unavailable => native_output_witness_stop(worth_relational::facade::mvcc::CompanionPreflightStop::ManagedSourceNotPrepared),
    }
}
