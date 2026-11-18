//! Final support compares genuine post-own and complete candidate images before World.
//!
//! Staged settlement (#224): an authored stage's read is checked as of its own
//! position. A later change by a member that sequentially runs after the stage
//! (a later stage, or a producer owed by this stage or a later one) is what
//! one commit per stage would commit too. A change by a producer owed before
//! the stage means the stage read a value that was not yet settled; that refuses.
//! The stage's discovery support never settles; a verified output it read
//! settles only when such a producer re-derived that same route.
//! Producer readers stay strict. The #229 hold marks only a refusal where the
//! adjacent writer images show a later authored stage changed a fact read by
//! a producer derived before that stage: the producer's own decision read, or
//! the support of a verified output (prepared in the envelope or committed)
//! that no derivation after the stage replaces.
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
    producer_reader: bool,
    hold: &mut Option<Failure>,
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
                    fact.support_equal_after_own_effect(
                        execution, runtime, before, earlier, image,
                    )
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
                |writer| producer_reader && is_stage(writer),
                |earlier, image| {
                    compare(earlier, image).map(|same| (!same).then(|| fact.locator_identity()))
                },
                fact.locator_identity(),
                |locator| format!("a later coupled contribution changed decision support: {locator}"),
                hold,
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
    complete: &worth_relational::facade::runtime::RelationalPinnedRead<
        worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
    >,
    members: &[crate::domain_computation::primary_graph::provider::CoupledApplicationMember],
    hold: &mut Option<Failure>,
) -> Result<(), CoupledSupportPreparationDenial> {
    // Each retained read is attributed from its own observed image, so an
    // ordinary root without a coupled post-own pin validates the same way.
    for (source, position, support) in attempt.candidate_read_support() {
        let compare = |earlier: &Candidate, image: &Candidate| {
            provider.graph.with_runtime(execution, |runtime| {
                support.changed_fact(execution, runtime, before, earlier, image)
            })
        };
        match compare(support.pin(), complete) {
            Ok(None) => {}
            Ok(Some(fact)) => changed(
                attempt,
                *source,
                *position,
                members,
                support.pin(),
                |writer| settles_read_support(attempt, support, *position, members, writer),
                |writer| superseded_parent(support, members, writer),
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
                hold,
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
    support: &crate::domain_computation::primary_graph::provider::CandidateReadSupport,
    position: usize,
    members: &[crate::domain_computation::primary_graph::provider::CoupledApplicationMember],
    writer: &crate::domain_computation::primary_graph::provider::CoupledApplicationMember,
) -> bool {
    use crate::domain_computation::primary_graph::provider::CandidateReadSupport;
    if reader.coupled_authored_subject().is_none() || !writer.settles_after(position) {
        return false;
    }
    let CandidateReadSupport::Parent(parent) = support else {
        return false;
    };
    let derives = |member: &crate::domain_computation::primary_graph::provider::CoupledApplicationMember| {
        member.settles_after(position) && derives_parent(member, parent)
    };
    derives(writer)
        || (is_stage(writer)
            && members.iter().any(|member| member.position > writer.position && derives(member)))
}

fn derives_parent(
    member: &crate::domain_computation::primary_graph::provider::CoupledApplicationMember,
    parent: &crate::domain_computation::primary_graph::application_contribution::producer::VerifiedAppliedParent,
) -> bool {
    member
        .producer_route()
        .is_some_and(|(_, meaning)| parent.has_route(meaning))
}

fn is_stage(
    writer: &crate::domain_computation::primary_graph::provider::CoupledApplicationMember,
) -> bool {
    matches!(
        writer.kind,
        crate::domain_computation::primary_graph::provider::CoupledApplicationMemberKind::AuthoredStage { .. }
    )
}

/// A verified output is a derivation that read its support: one prepared in
/// this envelope or the committed one the reader adopted. An authored stage
/// changing that support supersedes it (#229) unless the same route is derived
/// again after the stage, which settles instead; one commit per stage would
/// derive it again in the stage's own commit. Discovery support cannot name a
/// superseded derivation, so it never carries the hold.
fn superseded_parent(
    support: &crate::domain_computation::primary_graph::provider::CandidateReadSupport,
    members: &[crate::domain_computation::primary_graph::provider::CoupledApplicationMember],
    writer: &crate::domain_computation::primary_graph::provider::CoupledApplicationMember,
) -> bool {
    let crate::domain_computation::primary_graph::provider::CandidateReadSupport::Parent(parent) =
        support
    else {
        return false;
    };
    is_stage(writer)
        && !members
            .iter()
            .any(|member| member.position > writer.position && derives_parent(member, parent))
}

/// Only a changed fact walks the later writers. Reuse the post-own pins already
/// held by the admitted writers; healthy facts pay no intermediate comparisons.
/// `compare` names the fact that differs between two adjacent images, so each
/// refusal names the fact its own writer changed. An authored reader accepts
/// changes made by members that sequentially run after it; any other change
/// refuses, attributed to its writer. `held` names the writers whose refused
/// change superseded an earlier derivation (#229). Such a refusal is only
/// provisional: the scan continues, any plain refusal wins, and the hold is
/// recorded in `hold` for the caller to report after everything else validates.
fn changed(
    reader: &WorthQueryPrimaryGraphApplicationAttempt,
    source: worth_relational::facade::identity::EntityId,
    position: usize,
    members: &[crate::domain_computation::primary_graph::provider::CoupledApplicationMember],
    observed: &Candidate,
    settles: impl Fn(&crate::domain_computation::primary_graph::provider::CoupledApplicationMember) -> bool,
    held: impl Fn(&crate::domain_computation::primary_graph::provider::CoupledApplicationMember) -> bool,
    compare: impl Fn(&Candidate, &Candidate) -> Result<
        Option<String>,
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    >,
    final_locator: String,
    detail: impl Fn(&str) -> String,
    hold: &mut Option<Failure>,
) -> Result<(), CoupledSupportPreparationDenial> {
    let (mut settled, mut provisional) = (false, None);
    let mut earlier = observed;
    for writer in members.iter().filter(|writer| writer.position > position) {
        let attempt = &writer
            .owned
            .as_ref()
            .expect("actual extracted writer")
            .attempt;
        let pin = attempt
            .coupled_after_own()
            .expect("actual writer post-own image");
        let answer = compare(earlier, pin);
        earlier = pin;
        match answer {
            Ok(None) => continue,
            Err(cause) => return require_answer(Err(cause), source, String::new),
            Ok(Some(_)) if settles(writer) => settled = true,
            Ok(Some(locator)) => {
                let reader = reader
                    .coupled_authored_subject()
                    .map(|(stage, binding)| format!("authored stage {} {binding}", stage.ordinal()))
                    .unwrap_or_else(|| format!("reader {source:?}"));
                let later = match &writer.kind {
                    crate::domain_computation::primary_graph::provider::CoupledApplicationMemberKind::AuthoredStage { identity, binding, .. } => format!("authored stage {} {binding}", identity.ordinal()),
                    crate::domain_computation::primary_graph::provider::CoupledApplicationMemberKind::Producer { producer, meaning } => format!("producer {producer} {}", meaning.family),
                };
                let failure = Failure::coupled_read_support(
                    source,
                    None,
                    format!("{reader}; later writer {later}: {}", detail(&locator)),
                )
                .with_coupled_writer(writer.source());
                if !held(writer) {
                    return Err(CoupledSupportPreparationDenial::Changed(failure));
                }
                provisional.get_or_insert(failure.held(
                    crate::domain_computation::WorthQueryCoupledSupportHold::ProducerSupersession,
                ));
            }
        }
    }
    if let Some(failure) = provisional {
        hold.get_or_insert(failure);
        return Ok(());
    }
    if settled {
        return Ok(());
    }
    require_answer(Ok(false), source, || {
        format!("{}; no admitted later writer could be attributed", detail(&final_locator))
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
