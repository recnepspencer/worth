//! One wave's required chain: the caller's own Ready first, then the dirty
//! records popped from the shared queue as frames.

use super::super::super::required_continuations::{
    resume_held_upstream, ContinuationCustody, HeldUpstream, RequiredContinuations,
};
use super::super::super::RequiredFreshOutcome;
use super::super::{
    WorthQueryAdmittedOutputDemand, WorthQueryOutputDemandAdvance, WorthQueryProducerOutputFamily,
};
use super::queued::RequiredQueueFrames;
#[macro_use]
mod held_resume;
mod requested_refusal;
use super::selection::committed_ready;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerCommitAuthority;

/// The caller's own chain runs first; its errors end the call.
/// Dirty required records popped from the shared queue then run as frames
/// with the rest of the request. A queue chain never promotes or settles the
/// caller: its item is acknowledged when the item itself is Current on this
/// wave and is otherwise held for a later advance.
#[allow(clippy::too_many_arguments)]
pub(super) fn drive_required_wave<'runtime, Schema, Family>(
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request_scope: &WorthQueryRequestScope,
    commit_authority: &WorthQueryProducerCommitAuthority,
    mut wave: RequiredWaveSelection<'runtime, Schema>,
    queue: &mut RequiredQueueFrames,
    frame_custody: &mut RequiredContinuations<Schema>,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<WorthQueryOutputDemandAdvance>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    // Only an actual consumed-output edge can extend this closure. Initial
    // downstream B/C rows are irrelevant while their producer A is settling.
    let mut stack = RequiredWaveStack::new();
    let mut current: Option<SelectedReadyReadmission> = None;
    let mut current_contacts = 0usize;
    let mut current_role = FrameRole::Reached;
    // Rows certified Current on this wave; a dependent of several outputs
    // is readmitted once each of its pending edges has resolved here.
    let mut resolved_on_wave = ResolvedOnWave::default();
    // A queue chain that cannot finish on this wave keeps its item for a
    // later advance; only budget exhaustion ends the queue work.
    macro_rules! hold_queue_frame {
        ($label:lifetime, $stop:expr) => {{
            queue.hold_frame($stop, admission)?;
            stack.frames.clear();
            current = None;
            continue $label
        }};
    }
    // A stop ends the row being certified or refreshed. One intrinsic to the
    // row is recorded there, so dependents pending on that row fail with it;
    // the row's next Current certification or refresh clears it.
    macro_rules! stopped {
        ($label:lifetime, $key:expr, $stop:expr) => {{
            let stop = $stop;
            runtime.output_demands.record_required_stop($key, &stop);
            if queue.active() {
                frame_custody.end_refused(&stop);
                hold_queue_frame!($label, Some(stop))
            }
            return Err(stop);
        }};
    }
    // The caller's outcome is decided; queue frames take the rest.
    macro_rules! finish_caller {
        ($label:lifetime, $outcome:expr) => {{
            if wave.target == RequiredWaveTarget::Requested {
                return Ok(Some($outcome));
            }
            queue.finish_caller($outcome);
            stack.frames.clear();
            current = None;
            current_role = FrameRole::Reached;
            resolved_on_wave.clear();
            continue $label
        }};
    }
    'required: loop {
        if current.is_none() && stack.frames.is_empty() && queue.draining() {
            let Some(frame) = queue.next_frame(runtime, &wave, admission)? else {
                return Ok(queue.take_caller());
            };
            current = Some(frame);
            current_contacts = 0;
            current_role = FrameRole::Reached;
        }
        // One fixed scheduling step borrows the selected pin. Actual stack
        // writes and continuation installation are charged by their owners.
        admission
            .charge_external_work(6)
            .map_err(|_| work_denial())?;
        let selected = current.as_ref().unwrap_or(&wave.anchor_ready);
        if wave.target == RequiredWaveTarget::Caller && !queue.active() {
            // A restored output the caller's own chain consumed takes the
            // caller's mode when no demand of it has advanced.
            selected.completion().advanced_in(commit_authority);
        }
        admission
            .charge_external_work(3)
            .map_err(|_| work_denial())?;
        let producer_contacts_in_this_demand = if wave.target == RequiredWaveTarget::Caller
            && !queue.active()
            && (current.is_none() || current_role == FrameRole::CallerSuccessor)
        {
            // Every certification reports the caller handle's lifetime count.
            // Queue and upstream executions are not this caller's producer.
            demand.producer_contacts_in_this_demand
        } else {
            current_contacts
        };
        // The typed executor may admit and execute Fresh inside this call.
        // Reserve its custody before dispatch, even if it returns Current.
        let custody = if queue.active() {
            &mut *frame_custody
        } else {
            &mut demand.required_continuations
        };
        let slot = custody.prepare_slot(&runtime.output_demands, admission)?;
        let installation_work = slot.installation_work();
        let mut installation = admission
            .reserve_external_work(installation_work)
            .map_err(admission_denial)?;
        let resolved = resolved_on_wave.view(slot.entries());
        let caller_execution = wave.target == RequiredWaveTarget::Caller
            && !queue.active()
            && stack.frames.is_empty()
            && (current.is_none() || current_role == FrameRole::CallerSuccessor);
        let refresh_permission = super::super::refresh::permit_refresh(
            if caller_execution {
                demand.admission_kind
            } else {
                crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind::Required
            },
            Family::IDENTITY,
        );
        let mut result = certify_required_ready(
            runtime,
            principal,
            request_scope,
            &wave,
            selected,
            (current.is_none() && wave.target == RequiredWaveTarget::Caller)
                .then_some(demand.installed_entry.as_ref()),
            resolved.as_ref(),
            producer_contacts_in_this_demand,
            refresh_permission,
            installation.admission(),
        );
        if let (true, Ok(RequiredWaveStep::Fresh(progress))) = (caller_execution, &mut result) {
            // Account before installation, refusal, or any custody disposal.
            progress.attribute_to_caller(&mut demand.producer_contacts_in_this_demand);
        }
        installation
            .settle(if matches!(&result, Ok(RequiredWaveStep::Fresh(_))) {
                installation_work
            } else {
                0
            })
            .map_err(admission_denial)?;
        let result = match result {
            Ok(result) => result,
            Err(stop) => {
                drop(slot);
                stopped!('required, selected.key(), stop)
            }
        };
        match result {
            RequiredWaveStep::Current(settlement) => {
                drop(slot);
                runtime.output_demands.clear_required_stop(selected.key());
                if wave.target == RequiredWaveTarget::Requested
                    && stack.frames.is_empty()
                    && (current.is_none() || current_role == FrameRole::CallerSuccessor)
                {
                    return Ok(Some(WorthQueryOutputDemandAdvance::Settled(settlement)));
                }
                if wave.target == RequiredWaveTarget::Caller
                    && stack.frames.is_empty()
                    && !queue.active()
                    && super::caller::finish_current_caller(
                        runtime,
                        demand,
                        super::caller::CurrentCaller {
                            anchor_ready: &wave.anchor_ready,
                            selected,
                            continues_caller: current_role == FrameRole::CallerSuccessor,
                            caller_current: current.is_none(),
                            current_contacts: producer_contacts_in_this_demand,
                        },
                        admission,
                    )?
                {
                    finish_caller!('required, WorthQueryOutputDemandAdvance::Settled(settlement))
                }
                if current.is_some() {
                    // Admit the reached frame transfer before moving its pin.
                    admission
                        .charge_external_work(2)
                        .map_err(|_| work_denial())?;
                }
                if let Some(resolved) = current.take() {
                    let resolved_role = std::mem::replace(&mut current_role, FrameRole::Reached);
                    if stack.frames.is_empty() && queue.active() {
                        // The queued record itself is Current on this wave.
                        queue.discharge_frame(admission)?;
                        // A refreshed successor stays the resolved predecessor for
                        // later frames: their consumed edges still name the old
                        // identity, and each match is proven exactly. Any other
                        // discharged chain resolves nothing for the next one.
                        resolved_on_wave.clear();
                        if resolved_role != FrameRole::Reached {
                            resolved_on_wave.push(resolved, settlement, admission)?;
                        }
                        continue;
                    }
                    resolved_on_wave.push(resolved, settlement, admission)?;
                    if let Some(downstream) = stack.pop() {
                        let (ready, role, contacts) = downstream.into_parts();
                        current = Some(ready);
                        current_contacts = contacts;
                        current_role = role;
                    }
                    // An empty stack returns to the retained caller Ready.
                    continue;
                }
                finish_caller!('required, WorthQueryOutputDemandAdvance::Settled(settlement))
            }
            RequiredWaveStep::Upstream(upstream) => {
                drop(slot);
                if super::cycles::upstream_cycles(
                    &upstream,
                    selected,
                    &wave.anchor_ready,
                    &resolved_on_wave,
                    admission,
                )? {
                    if queue.active() {
                        hold_queue_frame!('required, None)
                    }
                    finish_caller!('required, WorthQueryOutputDemandAdvance::Pending)
                }
                if !stack.suspend_current(
                    &mut current,
                    current_role,
                    current_contacts,
                    admission,
                )? {
                    if queue.active() {
                        hold_queue_frame!('required, None)
                    }
                    finish_caller!('required, WorthQueryOutputDemandAdvance::Pending)
                }
                current = Some(upstream);
                current_contacts = 0;
                current_role = FrameRole::Reached;
            }
            RequiredWaveStep::Fresh(progress) => {
                let successor_role = if stack.frames.is_empty()
                    && !queue.active()
                    && (current.is_none() || current_role == FrameRole::CallerSuccessor)
                {
                    FrameRole::CallerSuccessor
                } else {
                    FrameRole::Successor
                };
                match slot.install(progress) {
                    RequiredFreshOutcome::Advanced => {}
                    RequiredFreshOutcome::Refused(mut stop) => {
                        match requested_refusal::resolve(
                            runtime,
                            custody,
                            &mut stop,
                            requested_refusal::RequestedDependency {
                                selected,
                                wave: &wave,
                                resolved: &resolved_on_wave,
                            },
                            admission,
                        )? {
                            requested_refusal::RequestedRefusal::Ready(upstream) => {
                                if !stack.suspend_current(
                                    &mut current,
                                    current_role,
                                    current_contacts,
                                    admission,
                                )? {
                                    if queue.active() {
                                        hold_queue_frame!('required, None)
                                    }
                                    finish_caller!('required, WorthQueryOutputDemandAdvance::Pending)
                                }
                                current = Some(upstream);
                                current_contacts = 0;
                                current_role = FrameRole::Reached;
                                continue 'required;
                            }
                            requested_refusal::RequestedRefusal::Cycle => {
                                if queue.active() {
                                    hold_queue_frame!('required, None)
                                }
                                finish_caller!('required, WorthQueryOutputDemandAdvance::Pending)
                            }
                            requested_refusal::RequestedRefusal::Held(head) => {
                                resume_held!('required, head;
                                    runtime, principal, request_scope;
                                    wave, resolved_on_wave, queue, frame_custody;
                                    demand, admission;
                                    hold_queue_frame, finish_caller, stopped)
                            }
                            requested_refusal::RequestedRefusal::Unavailable => {}
                        }
                        stopped!('required, selected.key(), stop)
                    }
                }
                loop {
                    // Rejoin the actual successor after each real
                    // Published/Delivered stage. A deferred stage leaves its
                    // checkpoint installed and returns Pending without
                    // spinning or reexecuting.
                    admission
                        .charge_external_work(2)
                        .map_err(|_| work_denial())?;
                    let custody = if queue.active() {
                        &mut *frame_custody
                    } else {
                        &mut demand.required_continuations
                    };
                    let successor = custody
                        .last()
                        .expect("the prepared slot installed one successor");
                    if let Some(ready) = runtime
                        .output_demands
                        .interest_ready_readmission(successor.interest(), admission)?
                    {
                        runtime
                            .output_demands
                            .clear_required_stop(successor.interest().key());
                        current_contacts = if wave.target == RequiredWaveTarget::Caller
                            && successor_role == FrameRole::CallerSuccessor
                        {
                            demand.producer_contacts_in_this_demand
                        } else {
                            successor.producer_contacts()
                        };
                        if committed_ready(&ready, admission)? {
                            wave = reselect_required_wave(runtime, wave, admission)?;
                            resolved_on_wave.clear();
                            queue.wave_moved();
                        }
                        current = Some(ready);
                        current_role = successor_role;
                        continue 'required;
                    }
                    admission
                        .charge_external_work(1)
                        .map_err(|_| work_denial())?;
                    let successor = custody
                        .last_mut()
                        .expect("the prepared slot installed one successor");
                    let progressed =
                        match successor.advance_checkpoint(runtime, request_scope, admission) {
                            Ok(progressed) => progressed,
                            Err(stop) => stopped!('required, successor.interest().key(), stop),
                        };
                    if !progressed {
                        if queue.active() {
                            hold_queue_frame!('required, None)
                        }
                        finish_caller!('required, WorthQueryOutputDemandAdvance::Pending)
                    }
                }
            }
            RequiredWaveStep::Held(head) => {
                drop(slot);
                resume_held!('required, head;
                                    runtime, principal, request_scope;
                                    wave, resolved_on_wave, queue, frame_custody;
                                    demand, admission;
                                    hold_queue_frame, finish_caller, stopped)
            }
            RequiredWaveStep::Pending => {
                if queue.active() {
                    hold_queue_frame!('required, None)
                }
                finish_caller!('required, WorthQueryOutputDemandAdvance::Pending)
            }
        }
    }
}
