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
    rebase: Option<
        &super::super::super::evidence_seal::postcommit_currentness::PreparedSourceFactRebase,
    >,
) -> Result<(), CoupledSupportPreparationDenial> {
    let after_own = attempt
        .coupled_after_own()
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
            if let Some(rebase) = rebase {
                rebase.support_equal_after_own_effect(
                    execution, runtime, before, &fact, after_own, complete,
                )
            } else {
                fact.support_equal_after_own_effect(execution, runtime, before, after_own, complete)
            }
        });
        require_answer(answer, source, || {
            format!(
                "a later coupled contribution changed decision support: {}",
                fact.locator_identity()
            )
        })?;
    }
    Ok(())
}
pub(super) fn require_read_support(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
    before: &SnapshotHandle,
    complete: &worth_relational::facade::runtime::RelationalPinnedRead<
        worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
    >,
) -> Result<(), CoupledSupportPreparationDenial> {
    for (source, support) in attempt.candidate_read_support() {
        let answer = provider.graph.with_runtime(execution, |runtime| {
            support.changed_fact(execution, runtime, before, complete)
        });
        match answer {
            Ok(None) => {}
            Ok(Some(fact)) => require_answer(Ok(false), *source, || {
                format!(
                    "{}: a later coupled contribution changed {}: {}",
                    support.subject(),
                    support.description(),
                    fact.locator_identity()
                )
            })?,
            Err(cause) => require_answer(Err(cause), *source, || support.subject().to_owned())?,
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
) -> Result<(), CoupledSupportPreparationDenial> {
    let failure = match answer {
        Ok(true) => return Ok(()),
        Ok(false) => Failure::coupled_read_support(source, None, changed()),
        Err(cause) => {
            if let Some(cause) = cause.relational_execution_cause() {
                return Err(CoupledSupportPreparationDenial::Execution(cause));
            }
            if let Some(cause) = cause.allocation_denial() {
                return Err(CoupledSupportPreparationDenial::Allocation(cause.clone()));
            }
            Failure::coupled_read_support(source, Some(cause.kind()), cause.to_string())
        }
    };
    Err(CoupledSupportPreparationDenial::Changed(failure))
}

impl CoupledSupportPreparationDenial {
    pub(in super::super) fn into_memory_stop(
        self,
    ) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
        use crate::domain_computation::WorthQueryProviderSessionCommitStop as Stop;
        match self {
            Self::Execution(cause) => crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_stop(cause, None),
            Self::Allocation(cause) => Stop::PreEffectDenied(WorthQueryProviderSessionFailure::allocation_denied(cause, "coupled declared read support")),
            Self::Changed(failure) => Stop::PreEffectDenied(
                super::super::failure("coupled candidate decision support refused before publication")
                    .with_candidate_support_failure(failure),
            ),
        }
    }
}
