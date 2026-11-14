//! Final support compares genuine post-own and complete candidate images before World.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationDecisionFact as Fact;
use crate::domain_computation::WorthQueryInvariantExecutionFailure as Failure;
type Candidate = worth_relational::facade::runtime::RelationalPinnedRead<
    worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
>;
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
    position: usize,
    members: &[crate::domain_computation::primary_graph::provider::CoupledApplicationMember],
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
        let compare = |image: &Candidate| {
            provider.graph.with_runtime(execution, |runtime| {
                if let Some(rebase) = rebase {
                    rebase.support_equal_after_own_effect(
                        execution, runtime, before, &fact, after_own, image,
                    )
                } else {
                    fact.support_equal_after_own_effect(
                        execution, runtime, before, after_own, image,
                    )
                }
            })
        };
        let answer = compare(complete);
        if matches!(answer, Ok(false)) {
            return changed(
                attempt,
                source,
                position,
                members,
                compare,
                format!(
                    "a later coupled contribution changed decision support: {}",
                    fact.locator_identity(),
                ),
            );
        }
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
    members: &[crate::domain_computation::primary_graph::provider::CoupledApplicationMember],
) -> Result<(), CoupledSupportPreparationDenial> {
    for (source, position, support) in attempt.candidate_read_support() {
        let answer = provider.graph.with_runtime(execution, |runtime| {
            support.changed_fact(execution, runtime, before, complete)
        });
        match answer {
            Ok(None) => {}
            Ok(Some(fact)) => changed(
                attempt,
                *source,
                *position,
                members,
                |image| {
                    provider.graph.with_runtime(execution, |runtime| {
                        support
                            .changed_fact(execution, runtime, before, image)
                            .map(|fact| fact.is_none())
                    })
                },
                format!(
                    "{}: a later coupled contribution changed {}: {}",
                    support.subject(),
                    support.description(),
                    fact.locator_identity()
                ),
            )?,
            Err(cause) => require_answer(Err(cause), *source, || support.subject().to_owned())?,
        }
    }
    Ok(())
}

/// Only a real final refusal needs attribution. Reuse the post-own pins already
/// held by the admitted writers; healthy facts pay no intermediate comparisons.
fn changed(
    reader: &WorthQueryPrimaryGraphApplicationAttempt,
    source: worth_relational::facade::identity::EntityId,
    position: usize,
    members: &[crate::domain_computation::primary_graph::provider::CoupledApplicationMember],
    compare: impl Fn(
        &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    ) -> Result<
        bool,
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    >,
    detail: String,
) -> Result<(), CoupledSupportPreparationDenial> {
    for writer in members.iter().filter(|writer| writer.position > position) {
        let attempt = &writer
            .owned
            .as_ref()
            .expect("actual extracted writer")
            .attempt;
        let pin = attempt
            .coupled_after_own()
            .expect("actual writer post-own image");
        match compare(pin) {
            Ok(true) => continue,
            Err(cause) => return require_answer(Err(cause), source, || detail.clone()),
            Ok(false) => {
                let earlier = reader
                    .coupled_authored_subject()
                    .map(|(stage, binding)| format!("authored stage {} {binding}", stage.ordinal()))
                    .unwrap_or_else(|| format!("reader {source:?}"));
                let later = match &writer.kind {
                    crate::domain_computation::primary_graph::provider::CoupledApplicationMemberKind::AuthoredStage { identity, binding, .. } => format!("authored stage {} {binding}", identity.ordinal()),
                    crate::domain_computation::primary_graph::provider::CoupledApplicationMemberKind::Producer { producer, meaning } => format!("producer {producer} {}", meaning.family),
                };
                return Err(CoupledSupportPreparationDenial::Changed(
                    Failure::coupled_read_support(
                        source,
                        None,
                        format!("{earlier}; later writer {later}: {detail}"),
                    )
                    .with_coupled_writer(writer.source()),
                ));
            }
        }
    }
    require_answer(Ok(false), source, || {
        format!("{detail}; no admitted later writer could be attributed")
    })
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
