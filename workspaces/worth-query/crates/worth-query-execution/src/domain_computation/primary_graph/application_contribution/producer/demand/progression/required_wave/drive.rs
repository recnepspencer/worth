//! Collect dependency-ready required members, then compute and apply one wave.
use super::super::super::required_continuations::resume_held_upstream;
use super::super::super::required_continuations::resume_owned_refresh;
use super::super::super::required_continuations::ContinuationCustody;
use super::super::super::required_continuations::HeldUpstream;
use super::super::super::required_continuations::OwnedRefreshStop;
use super::super::super::required_continuations::RequiredContinuations;
use super::super::WorthQueryAdmittedOutputDemand;
use super::super::WorthQueryOutputDemandAdvance;
use super::super::WorthQueryProducerOutputFamily;
use super::frontier::DependencySchedule;
use super::frontier::MemberInput;
use super::frontier::MemberState;
use super::frontier::RequiredFrontier;
use super::frontier::RequiredUpstream;
use super::queued::QueuedRequiredRoots;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerCommitAuthority;
pub(super) mod canonical_apply;
mod member_refusal;
mod member_stop;
use member_refusal::RefusalRow;
use member_refusal::RefusingMember;
mod frozen_collection;
mod readiness_collection;
mod requested_refusal;
mod retained_refresh;
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
) -> Result<Option<WorthQueryOutputDemandAdvance>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    runtime
        .output_demands
        .record_selection_state(wave.is_frozen());
    let mut frontier = RequiredFrontier::new();
    frontier.insert(wave.anchor_ready.retain()?, true, None)?;
    let mut resolved = ResolvedOnWave::default();
    loop {
        // The caller's own selection closes its membership handoff before
        // unrelated queue claims join. Actual pending consumed edges collect
        // their issued prerequisites together during readiness collection.
        let caller_was_current = queue.draining();
        if wave.target == RequiredWaveTarget::Caller && caller_was_current && !wave.is_frozen() {
            while let Some((queued, claim)) = queue.next_claim(runtime, &wave)? {
                frontier.insert_claim(claim, queued)?;
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
        )?;
        if let Some(delivery) = result.delivery {
            wave = wave.after_awaited_delivery(execution, runtime, delivery)?;
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
                canonical_apply::finish(runtime, demand, &wave, &mut frontier, queue)?
            {
                return Ok(Some(outcome));
            }
            continue;
        }
        if wave.target == RequiredWaveTarget::Caller && !queue.draining() {
            match canonical_apply::accept_current_caller(runtime, demand, &wave, &mut frontier)? {
                canonical_apply::CallerAcceptance::Accepted(proof) => queue.finish_caller(proof),
                canonical_apply::CallerAcceptance::Deferred => {}
            }
        }
        if !caller_was_current && queue.draining() {
            resolved.clear();
        }
        if result.moved {
            queue.wave_moved();
            wave = wave.reselect(execution, runtime)?;
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
            )? {
                requested_refusal::RequestedRefusal::Ready(ready) => {
                    match frontier.require_upstream(ready, member.caller)? {
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
                    .end(stop, recording, queue)?;
                    if let Some(outcome) = ended.finish(|| {
                        canonical_apply::finish(runtime, demand, &wave, &mut frontier, queue)
                    }) {
                        return outcome;
                    }
                }
            }
        }
        if result.moved {
            for member in &mut frontier.members {
                if retained_refresh::preserves_selection(
                    member,
                    &demand.required_continuations,
                    queue_custody,
                )? {
                    continue;
                }
                if !member.resolved_successor
                    && matches!(
                        member.state,
                        MemberState::Inspect
                            | MemberState::Waiting(_)
                            | MemberState::WaitingDeclaration(_)
                            | MemberState::Held(_)
                            | MemberState::RetainedRefresh
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
        // A Fresh attempt that requested an upstream already owns the
        // predecessor's replacement. Resume that typed owner, not the stopped
        // Ready pin from before its admission. No prepared plan remains here.
        let refresh = (!wave.is_frozen())
            .then(|| {
                frontier
                    .members
                    .iter()
                    .position(|member| matches!(member.state, MemberState::RetainedRefresh))
            })
            .flatten();
        if let Some(index) = refresh {
            let custody = retained_refresh::custody(
                frontier.members[index].caller,
                wave.target,
                &mut demand.required_continuations,
                &mut demand.producer_contacts_in_this_demand,
                &mut *queue_custody,
            );
            let outcome = resume_owned_refresh(
                phase,
                runtime,
                principal,
                request,
                wave.branch,
                custody,
                frontier.members[index].ready.selected(),
                performed,
            );
            match outcome {
                Ok(Some(HeldUpstream::Ready(ready))) => {
                    let moved = super::selection::committed_ready(&ready)?;
                    let member = &mut frontier.members[index];
                    member.ready.bind_inspected_native(ready);
                    member.resolved_successor = true;
                    member.caller_successor |= index == 0;
                    member.state = MemberState::Inspect;
                    if moved {
                        wave = wave.reselect(execution, runtime)?;
                        resolved.clear();
                        queue.wave_moved();
                    }
                }
                Ok(Some(HeldUpstream::GaveBack) | None) => {
                    frontier.members[index].state = MemberState::Inspect
                }
                Ok(Some(HeldUpstream::Unfinished)) => {
                    frontier.members[index].state = MemberState::Pending
                }
                Err(OwnedRefreshStop::Selection(stop)) => return Err(stop),
                Err(OwnedRefreshStop::Recorded(stop)) => {
                    let ended = RefusingMember::from_driver(
                        &runtime.output_demands,
                        &mut frontier.members[index],
                        RefusalRow::Member,
                        &mut *queue_custody,
                    )
                    .end(
                        stop,
                        member_stop::StopRecording::successor(),
                        queue,
                    )?;
                    if let Some(outcome) = ended.finish(|| {
                        canonical_apply::finish(runtime, demand, &wave, &mut frontier, queue)
                    }) {
                        return outcome;
                    }
                }
            }
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
            let custody = retained_refresh::custody(
                frontier.members[index].caller,
                wave.target,
                &mut demand.required_continuations,
                &mut demand.producer_contacts_in_this_demand,
                &mut *queue_custody,
            );
            match resume_held_upstream(
                phase,
                runtime,
                principal,
                request,
                wave.branch,
                custody,
                &head,
                performed,
            ) {
                Ok(HeldUpstream::Ready(ready)) => {
                    runtime.output_demands.clear_required_stop(&head);
                    if super::selection::committed_ready(&ready)? {
                        wave = wave.reselect(execution, runtime)?;
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
                    .end(stop, member_stop::StopRecording::member(), queue)?;
                    if let Some(outcome) = ended.finish(|| {
                        canonical_apply::finish(runtime, demand, &wave, &mut frontier, queue)
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
            canonical_apply::finish(runtime, demand, &wave, &mut frontier, queue)?
        {
            return Ok(Some(outcome));
        }
    }
}
