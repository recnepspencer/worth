//! Collect dependency-ready required members, then compute and apply one wave.
use super::super::super::required_continuations::{
    resume_held_upstream, ContinuationCustody, HeldUpstream, RequiredContinuations,
};
use super::super::{
    WorthQueryAdmittedOutputDemand, WorthQueryOutputDemandAdvance, WorthQueryProducerOutputFamily,
};
use super::frontier::{DependencySchedule, MemberState, RequiredFrontier};
use super::queued::QueuedRequiredRoots;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerCommitAuthority;
mod canonical_apply;
mod readiness_collection;
mod requested_refusal;
mod wave_execution;

#[allow(clippy::too_many_arguments)]
pub(super) fn drive_required_wave<'runtime, Schema, Family>(
    phase: &WorthQueryAdvancementPhase<'_>,
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    authority: &WorthQueryProducerCommitAuthority,
    mut wave: RequiredWaveSelection<'runtime, Schema>,
    queue: &mut QueuedRequiredRoots,
    queue_custody: &mut RequiredContinuations<Schema>,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<WorthQueryOutputDemandAdvance>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    let mut frontier = RequiredFrontier::new();
    frontier.insert(wave.anchor_ready.retain(admission)?, true, None, admission)?;
    let mut resolved = ResolvedOnWave::default();
    loop {
        // The caller's own selection closes its membership handoff before
        // unrelated queue claims join. Actual pending consumed edges collect
        // their issued prerequisites together during readiness collection.
        let caller_was_current = queue.draining();
        if wave.target == RequiredWaveTarget::Caller && caller_was_current {
            while let Some((queued, ready)) = queue.next_root(runtime, &wave, admission)? {
                frontier.insert(ready, false, Some(queued), admission)?;
            }
        }
        let result = wave_execution::run::<Schema, Family>(
            phase,
            runtime,
            demand,
            principal,
            request,
            authority,
            &wave,
            &mut frontier,
            &mut resolved,
            queue_custody,
            admission,
        )?;
        let mut selection_changed = false;
        for member in &mut frontier.members {
            if let Some(index) = member.queued {
                if !member.resolved_successor
                    && !queue.retains_selection(runtime, index, &member.ready, admission)?
                {
                    member.queued = None;
                    member.state = MemberState::Pending;
                    selection_changed = true;
                    continue;
                }
            }
            if matches!(member.state, MemberState::Current(_)) {
                if let Some(index) = member.queued.take() {
                    queue.acknowledge_root(index, admission)?;
                }
            }
        }
        if wave.target == RequiredWaveTarget::Caller && !queue.draining() {
            if let Some(outcome) = canonical_apply::accept_current_caller(
                runtime, demand, &wave, &frontier, admission,
            )? {
                queue.finish_caller(outcome);
            }
        }
        if !caller_was_current && queue.draining() {
            resolved.clear();
        }
        if result.moved {
            queue.wave_moved();
            wave = reselect_required_wave(runtime, wave, admission)?;
            resolved.clear();
            for member in &mut frontier.members {
                member.dependency_schedule = DependencySchedule::OwnSelection;
            }
        }
        if let Some((index, mut stop)) = result.stopped {
            let member = &frontier.members[index];
            let custody = if member.caller {
                &mut demand.required_continuations
            } else {
                &mut *queue_custody
            };
            match requested_refusal::resolve(
                runtime,
                custody,
                &mut stop,
                requested_refusal::RequestedDependency {
                    selected: &member.ready,
                    wave: &wave,
                    resolved: &resolved,
                },
                admission,
            )? {
                requested_refusal::RequestedRefusal::Ready(ready) => {
                    let upstream = frontier.require_upstream(ready, member.caller, admission)?;
                    frontier.wait_for(index, upstream);
                    continue;
                }
                requested_refusal::RequestedRefusal::Held(head) => {
                    frontier.members[index].state = MemberState::Held(head);
                    continue;
                }
                requested_refusal::RequestedRefusal::Cycle => {
                    frontier.members[index].state = MemberState::Pending;
                }
                requested_refusal::RequestedRefusal::Unavailable => {
                    runtime
                        .output_demands
                        .record_required_stop(member.ready.key(), &stop);
                    if member.caller {
                        return Err(stop);
                    }
                    queue_custody.end_refused(&stop);
                    frontier.members[index].state = MemberState::Pending;
                    if matches!(
                        stop.kind(),
                        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
                            | WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
                    ) {
                        return canonical_apply::finish(
                            runtime, demand, &wave, &frontier, queue, admission,
                        );
                    }
                    if let Some(queued) = frontier.members[index].queued.take() {
                        queue.hold_root(queued, Some(stop), admission)?;
                    }
                }
            }
        }
        if selection_changed
            || result.prepared
            || frontier.awaken()
            || (!caller_was_current
                && wave.target == RequiredWaveTarget::Caller
                && queue.draining())
        {
            continue;
        }
        // A held successor is resumed only after every prepared member has
        // left the selected basis. It retains the actual issued demand.
        let held =
            frontier
                .members
                .iter()
                .enumerate()
                .find_map(|(i, member)| match &member.state {
                    MemberState::Held(key) => Some((i, key.clone())),
                    _ => None,
                });
        let resumed = held.is_some();
        if let Some((index, head)) = held {
            let custody =
                if frontier.members[index].caller && wave.target == RequiredWaveTarget::Caller {
                    ContinuationCustody::Caller(
                        &mut demand.required_continuations,
                        &mut demand.producer_contacts_in_this_demand,
                    )
                } else {
                    ContinuationCustody::Queue(queue_custody)
                };
            match resume_held_upstream(
                phase,
                runtime,
                principal,
                request,
                wave.branch,
                custody,
                &head,
                admission,
            ) {
                Ok(HeldUpstream::Ready(ready)) => {
                    runtime.output_demands.clear_required_stop(&head);
                    if super::selection::committed_ready(&ready, admission)? {
                        wave = reselect_required_wave(runtime, wave, admission)?;
                        resolved.clear();
                        queue.wave_moved();
                    }
                    frontier.members[index].state = MemberState::Inspect;
                    continue;
                }
                Ok(HeldUpstream::GaveBack) => {
                    frontier.members[index].state = MemberState::Inspect;
                    continue;
                }
                Ok(HeldUpstream::Unfinished) => {
                    frontier.members[index].state = MemberState::Pending
                }
                Err(stop) if frontier.members[index].caller => return Err(stop),
                Err(stop) => {
                    runtime.output_demands.record_required_stop(&head, &stop);
                    frontier.members[index].state = MemberState::Pending;
                }
            }
        }
        if resumed {
            continue;
        }
        return canonical_apply::finish(runtime, demand, &wave, &frontier, queue, admission);
    }
}
