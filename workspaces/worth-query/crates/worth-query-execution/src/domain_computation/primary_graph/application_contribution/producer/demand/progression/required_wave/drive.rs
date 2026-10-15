//! Collect dependency-ready required members, then compute and apply one wave.
use super::super::super::required_continuations::{
    resume_held_upstream, ContinuationCustody, HeldUpstream, RequiredContinuations,
};
use super::super::{
    WorthQueryAdmittedOutputDemand, WorthQueryOutputDemandAdvance, WorthQueryProducerOutputFamily,
};
use super::frontier::{
    DependencySchedule, MemberInput, MemberState, RequiredFrontier, RequiredUpstream,
};
use super::queued::QueuedRequiredRoots;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerCommitAuthority;
pub(super) mod canonical_apply;
mod member_refusal;
mod member_stop;
use member_refusal::{RefusalRow, RefusingMember};
mod frozen_collection;
mod readiness_collection;
mod requested_refusal;
mod wave_execution;

#[allow(clippy::too_many_arguments)]
pub(super) fn drive_required_wave<'runtime, Schema, Family>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &WorthQueryAdvancementPhase<'_>,
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    authority: &WorthQueryProducerCommitAuthority,
    mut wave: RequiredWaveSelection<'runtime, Schema>,
    queue: &mut QueuedRequiredRoots,
    queue_custody: &mut RequiredContinuations<Schema>,
    performed: &std::cell::RefCell<performed::PerformedMembers>,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<WorthQueryOutputDemandAdvance>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    runtime
        .output_demands
        .record_selection_state(wave.is_frozen());
    let mut frontier = RequiredFrontier::new();
    frontier.insert(wave.anchor_ready.retain(admission)?, true, None, admission)?;
    let mut resolved = ResolvedOnWave::default();
    loop {
        // The caller's own selection closes its membership handoff before
        // unrelated queue claims join. Actual pending consumed edges collect
        // their issued prerequisites together during readiness collection.
        let caller_was_current = queue.draining();
        if wave.target == RequiredWaveTarget::Caller && caller_was_current && !wave.is_frozen() {
            while let Some((queued, claim)) = queue.next_claim(runtime, &wave, admission)? {
                frontier.insert_claim(claim, queued, admission)?;
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
            queue,
            performed,
            admission,
        )?;
        if let Some(delivery) = result.delivery {
            wave = wave.after_awaited_delivery(execution, runtime, delivery, admission)?;
            frontier.members[0].state = MemberState::Inspect;
            if !wave.is_frozen() {
                continue;
            }
        }
        if result.deferred {
            wave.freeze();
        }
        if wave.target == RequiredWaveTarget::Requested
            && matches!(frontier.members[0].state, MemberState::Current(_))
        {
            if let Some(outcome) =
                canonical_apply::finish(runtime, demand, &wave, &mut frontier, queue, admission)?
            {
                return Ok(Some(outcome));
            }
            continue;
        }
        if wave.target == RequiredWaveTarget::Caller && !queue.draining() {
            match canonical_apply::accept_current_caller(
                runtime,
                demand,
                &wave,
                &mut frontier,
                admission,
            )? {
                canonical_apply::CallerAcceptance::Accepted(proof) => queue.finish_caller(proof),
                canonical_apply::CallerAcceptance::Deferred => {}
            }
        }
        if !caller_was_current && queue.draining() {
            resolved.clear();
        }
        if result.moved {
            queue.wave_moved();
            wave = wave.reselect(execution, runtime, admission)?;
            resolved.clear();
            for member in &mut frontier.members {
                member.dependency_schedule = DependencySchedule::OwnSelection;
            }
        }
        for failure in result.queued_stops.into_iter().chain(result.stopped) {
            let (index, stop, recording) = failure.into_parts();
            let mut stop = member_refusal::member_stop(stop)?;
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
                    selected: member.ready.selected(),
                    wave: &wave,
                    resolved: &resolved,
                },
                admission,
            )? {
                requested_refusal::RequestedRefusal::Ready(ready) => {
                    match frontier.require_upstream(ready, member.caller, admission)? {
                        RequiredUpstream::Member(upstream) => {
                            frontier.wait_for(index, upstream);
                        }
                        RequiredUpstream::HeldCaller => {
                            frontier.members[index].state = MemberState::Pending;
                        }
                    }
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
                    let ended = RefusingMember::from_driver(
                        &runtime.output_demands,
                        &mut frontier.members[index],
                        RefusalRow::Member,
                        &mut *queue_custody,
                    )
                    .end(stop, recording, queue, admission)?;
                    if let Some(outcome) = ended.finish(|| {
                        canonical_apply::finish(
                            runtime,
                            demand,
                            &wave,
                            &mut frontier,
                            queue,
                            admission,
                        )
                    }) {
                        return outcome;
                    }
                }
            }
        }
        if result.moved {
            for member in &mut frontier.members {
                if !member.resolved_successor
                    && matches!(
                        member.state,
                        MemberState::Inspect
                            | MemberState::Waiting(_)
                            | MemberState::WaitingDeclaration(_)
                            | MemberState::Held(_)
                    )
                    && let MemberInput::Queued { ready, .. } = &mut member.ready
                    && let Some(index) = member.queued
                {
                    queue.discard_selection(index);
                    *ready = None;
                }
            }
        }
        if !wave.is_frozen()
            && (result.prepared
                || frontier.awaken()
                || (!caller_was_current
                    && wave.target == RequiredWaveTarget::Caller
                    && queue.draining()))
        {
            continue;
        }
        // A held successor is resumed only after every prepared member has
        // left the selected basis. It retains the actual issued demand.
        let held = if wave.is_frozen() {
            None
        } else {
            frontier
                .members
                .iter()
                .enumerate()
                .find_map(|(i, member)| match &member.state {
                    MemberState::Held(key) => Some((i, key.clone())),
                    _ => None,
                })
        };
        let resumed = held.is_some();
        if let Some((index, head)) = held {
            let custody =
                if frontier.members[index].caller && wave.target == RequiredWaveTarget::Caller {
                    ContinuationCustody::Caller(
                        &mut demand.required_continuations,
                        &mut demand.producer_contacts_in_this_demand,
                    )
                } else if frontier.members[index].caller {
                    // A Requested target belongs to the parent demand without
                    // attributing the child's handler contacts to that parent.
                    ContinuationCustody::Queue(&mut demand.required_continuations)
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
                performed,
            ) {
                Ok(HeldUpstream::Ready(ready)) => {
                    runtime.output_demands.clear_required_stop(&head);
                    if super::selection::committed_ready(&ready, admission)? {
                        wave = wave.reselect(execution, runtime, admission)?;
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
                Err(stop) => {
                    let ended = RefusingMember::from_driver(
                        &runtime.output_demands,
                        &mut frontier.members[index],
                        RefusalRow::HeldHead(&head),
                        &mut *queue_custody,
                    )
                    .end(
                        stop,
                        member_stop::StopRecording::member(),
                        queue,
                        admission,
                    )?;
                    if let Some(outcome) = ended.finish(|| {
                        canonical_apply::finish(
                            runtime,
                            demand,
                            &wave,
                            &mut frontier,
                            queue,
                            admission,
                        )
                    }) {
                        return outcome;
                    }
                }
            }
        }
        if resumed {
            continue;
        }
        if wave.target == RequiredWaveTarget::Caller && !queue.draining() {
            // Pending decides the caller just as Current does. It leaves its
            // unfinished closure held while independent queue records proceed.
            queue.defer_caller();
            resolved.clear();
            if !wave.is_frozen() {
                continue;
            }
        }
        if let Some(outcome) =
            canonical_apply::finish(runtime, demand, &wave, &mut frontier, queue, admission)?
        {
            return Ok(Some(outcome));
        }
    }
}
