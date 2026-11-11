//! Final support compares genuine post-own and complete candidate images before World.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationDecisionFact as Fact;
use crate::domain_computation::WorthQueryInvariantExecutionFailure as Failure;

#[derive(Debug)]
pub(in super::super) enum CoupledSupportPreparationDenial {
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    Allocation(worth_execution::ExecutionAllocationDenial),
    Changed(Failure),
}
pub(super) fn require(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
    before: &SnapshotHandle,
    complete: &worth_relational::facade::runtime::RelationalPinnedRead<
        worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
    >,
    source: worth_relational::facade::identity::EntityId,
) -> Result<(), CoupledSupportPreparationDenial> {
    let after_own = attempt
        .coupled_preparation()
        .expect("actual registered writer")
        .after_own
        .as_ref()
        .expect("Native completed this writer before continuing");
    for fact in attempt.facts().values() {
        let fact = match fact {
            Fact::Application { fact, .. }
            | Fact::ObservedSource { fact }
            | Fact::ApplicationObservedSource { fact, .. } => fact,
            // Real typed writer authorization is independently revalidated at cutover.
            Fact::Principal(_) | Fact::Authorization { .. } => continue,
        };
        let answer = provider.graph.with_runtime(execution, |runtime| {
            fact.support_equal_after_own_effect(execution, runtime, before, after_own, complete)
        });
        let failure = match answer {
            Ok(true) => continue,
            Ok(false) => Failure::coupled_read_support(
                source,
                None,
                format!(
                    "a later coupled contribution changed decision support: {}",
                    fact.locator_identity()
                ),
            ),
            Err(cause) => {
                if let Some(cause) = cause.relational_execution_cause() {
                    return Err(CoupledSupportPreparationDenial::Execution(cause));
                }
                if let Some(cause) = cause.allocation_denial() {
                    return Err(CoupledSupportPreparationDenial::Allocation(cause.clone()));
                }
                Failure::coupled_read_support(
                    source,
                    Some(cause.kind()),
                    format!("{}: {}", fact.locator_identity(), cause),
                )
            }
        };
        return Err(CoupledSupportPreparationDenial::Changed(failure));
    }
    Ok(())
}

impl CoupledSupportPreparationDenial {
    pub(in super::super) fn into_memory_stop(
        self,
    ) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
        use crate::domain_computation::WorthQueryProviderSessionCommitStop as Stop;
        match self {
            Self::Execution(cause) => crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_stop(cause, None),
            Self::Allocation(cause) => Stop::PreEffectDenied(WorthQueryProviderSessionFailure::allocation_denied(cause)),
            Self::Changed(failure) => Stop::PreEffectDenied(
                super::super::failure("coupled candidate decision support refused before publication")
                    .with_candidate_support_failure(failure),
            ),
        }
    }
}
