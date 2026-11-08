//! Final support compares genuine post-own and complete candidate images before World.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationDecisionFact as Fact;
use crate::domain_computation::{
    WorthQueryInvariantExecutionFailure as Failure, WorthQueryProviderSessionCommitStop as Stop,
};
pub(super) fn require(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
    before: &SnapshotHandle,
    complete: &worth_relational::facade::runtime::RelationalPinnedRead<
        worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
    >,
    source: worth_relational::facade::identity::EntityId,
) -> Result<(), Stop> {
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
        require_answer(answer, source, || {
            format!(
                "a later coupled contribution changed decision support: {}",
                fact.locator_identity()
            )
        })?;
    }
    for support in &attempt
        .coupled_preparation()
        .expect("actual registered writer")
        .query_support
    {
        let answer = provider.graph.with_runtime(execution, |runtime| {
            support.changed_fact(execution, runtime, before, complete)
        });
        match answer {
            Ok(None) => {}
            Ok(Some(fact)) => require_answer(Ok(false), source, || {
                format!(
                    "{}: a later coupled contribution changed discovery support: {}",
                    support.query(),
                    fact.locator_identity()
                )
            })?,
            Err(cause) => require_answer(Err(cause), source, || support.query().to_owned())?,
        }
    }
    Ok(())
}

fn require_answer(
    answer: Result<
        bool,
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    >,
    source: worth_relational::facade::identity::EntityId,
    changed: impl FnOnce() -> String,
) -> Result<(), Stop> {
    let failure = match answer {
        Ok(true) => return Ok(()),
        Ok(false) => Failure::coupled_read_support(source, None, changed()),
        Err(cause) => {
            if let Some(cause) = cause.relational_execution_cause() {
                return Err(crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_stop(cause,None));
            }
            if let Some(cause) = cause.allocation_denial() {
                return Err(Stop::PreEffectDenied(
                    WorthQueryProviderSessionFailure::allocation_denied(cause.clone()),
                ));
            }
            Failure::coupled_read_support(source, Some(cause.kind()), cause.to_string())
        }
    };
    Err(Stop::PreEffectDenied(
        super::super::failure("coupled candidate decision support refused before publication")
            .with_candidate_support_failure(failure),
    ))
}
