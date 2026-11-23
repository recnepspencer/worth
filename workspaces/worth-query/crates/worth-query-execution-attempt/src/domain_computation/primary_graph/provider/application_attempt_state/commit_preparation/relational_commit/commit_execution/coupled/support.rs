//! Final support compares genuine post-own and complete candidate images before World.
//!
//! Staged settlement (#224): an authored stage's read is checked as of its own
//! position. A later change by a member that sequentially runs after the stage
//! (a later stage, or a producer owed by this stage or a later one) is what
//! one commit per stage would commit too. A change by a producer owed before
//! the stage means the stage read a value that was not yet settled; that refuses.
//! The stage's discovery support never settles; a verified output it read
//! settles only when such a producer re-derived that same route.
//! Producer readers stay strict. A retained read qualified use reopened is
//! checked only over its window (`window`); a root selection's stage window
//! by owed order. Whatever still differs refuses plainly (PR C Q4): no change
//! is held for an open issue.
use super::*;
mod changed;
pub mod court_doors_for_test;
mod window;
use crate::domain_computation::primary_graph::provider::{
    CandidateReadSupport, CoupledApplicationMember, CoupledApplicationMemberKind,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationDecisionFact as Fact;
use crate::domain_computation::WorthQueryInvariantExecutionFailure as Failure;
use changed::{changed, Round};
use window::superseded_window;
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
    complete: &Candidate,
    source: worth_relational::facade::identity::EntityId,
    rebase: Option<
        &super::super::super::evidence_seal::postcommit_currentness::PreparedSourceFactRebase,
    >,
    position: usize,
    members: &[CoupledApplicationMember],
) -> Result<(), CoupledSupportPreparationDenial> {
    for consumed in attempt.consumed_outputs() {
        if let Some(link) = consumed.applied()
            && !link.require_support(position, members, complete)
        {
            return Err(CoupledSupportPreparationDenial::Changed(
                Failure::coupled_read_support(
                    source,
                    None,
                    "same-candidate prerequisite lost its exact earlier registered writer",
                ),
            ));
        }
    }
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
        let compare = |earlier: &Candidate, image: &Candidate| {
            provider.graph.with_runtime(execution, |runtime| {
                if let Some(rebase) = rebase {
                    rebase.support_equal_after_own_effect(
                        execution, runtime, before, &fact, earlier, image,
                    )
                } else {
                    fact.support_equal_after_own_effect(execution, runtime, before, earlier, image)
                }
            })
        };
        let answer = compare(after_own, complete);
        if matches!(answer, Ok(false)) {
            changed(
                attempt,
                source,
                position,
                members,
                after_own,
                // Only an authored stage's own decision reads settle by position.
                |writer| {
                    attempt.coupled_authored_subject().is_some() && writer.settles_after(position)
                },
                |earlier, image| {
                    compare(earlier, image).map(|same| (!same).then(|| fact.locator_identity()))
                },
                fact.locator_identity(),
                |locator| {
                    format!("a later coupled contribution changed decision support: {locator}")
                },
            )?;
            continue;
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
    complete: &Candidate,
    members: &[CoupledApplicationMember],
) -> Result<(), CoupledSupportPreparationDenial> {
    // Each retained read is attributed from its own observed image, so an
    // ordinary root without a coupled post-own pin validates the same way.
    for retained in attempt.candidate_read_support() {
        let (source, position, support) = (&retained.source, &retained.position, &retained.support);
        let compare = |earlier: &Candidate, image: &Candidate| {
            provider.graph.with_runtime(execution, |runtime| {
                support.changed_fact(execution, runtime, before, earlier, image)
            })
        };
        // A root selection's stage window (#238) ends at the next authored
        // stage by owed order: a producer its own stage owes stays inside it
        // even when it physically appends after that stage.
        let stage_window = retained.window_end.filter(|_| retained.stage_window);
        if let (Some(invalidator), None) = (retained.window_end, stage_window) {
            superseded_window(*source, *position, support, invalidator, members, compare)?;
            continue;
        }
        match compare(support.pin(), complete) {
            Ok(None) => {}
            Ok(Some(fact)) => changed(
                attempt,
                *source,
                *position,
                members,
                support.pin(),
                |writer| match stage_window {
                    Some(stage) => writer.runs_from_stage(stage),
                    None => settles_read_support(attempt, support, *position, members, writer),
                },
                |earlier, image| {
                    compare(earlier, image).map(|fact| fact.map(|fact| fact.locator_identity()))
                },
                fact.locator_identity(),
                |locator| {
                    format!(
                        "{}: a later coupled contribution changed {}: {locator}",
                        support.subject(),
                        support.description(),
                    )
                },
            )?,
            Err(cause) => require_answer(Err(cause), *source, || support.subject().to_owned())?,
        }
    }
    Ok(())
}

/// Discovery support selected which outputs the candidate holds. No writer's
/// change to it is accepted, not even a producer owed later: nothing proves
/// the selection it governed was derived again. A verified output an authored
/// stage read settles only where the producer of that same route, owed at or
/// after the stage, derives it at or after the change: its own transition, or
/// a later authored stage's change to its support that it then re-derives. So
/// no stale output can commit.
fn settles_read_support(
    reader: &WorthQueryPrimaryGraphApplicationAttempt,
    support: &CandidateReadSupport,
    position: usize,
    members: &[CoupledApplicationMember],
    writer: &Round<'_>,
) -> bool {
    if reader.coupled_authored_subject().is_none() || !writer.settles_after(position) {
        return false;
    }
    let CandidateReadSupport::Parent(parent) = support else {
        return false;
    };
    let derives = |member: &CoupledApplicationMember| {
        member.settles_after(position) && derives_parent(member, parent.as_ref())
    };
    (writer.settles_after(position) && derives_parent(writer.member, parent.as_ref()))
        || (is_stage(writer.member)
            && members
                .iter()
                .any(|member| member.position > writer.position && derives(member)))
}

fn derives_parent(
    member: &CoupledApplicationMember,
    parent: &dyn crate::ports::CandidateReadSupportPort,
) -> bool {
    member
        .producer_route()
        .is_some_and(|(_, meaning)| parent.has_route(meaning))
}

fn is_stage(writer: &CoupledApplicationMember) -> bool {
    matches!(
        writer.kind,
        CoupledApplicationMemberKind::AuthoredStage { .. }
    )
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
