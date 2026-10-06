//! One wave's required chain: the caller's own Ready first, then the dirty
//! records popped from the shared queue as frames.

use super::super::super::required_continuations::{
    resume_held_upstream, HeldUpstream, RequiredContinuations,
};
use super::super::super::RequiredFreshOutcome;
use super::super::{
    WorthQueryAdmittedOutputDemand, WorthQueryOutputDemandAdvance, WorthQueryProducerOutputFamily,
};
use super::queued::RequiredQueueFrames;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerCommitAuthority;

/// The caller's own chain runs first; its errors end the call as before.
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
                frame_custody.end_refused(&runtime.output_demands, &stop, admission);
                hold_queue_frame!($label, Some(stop))
            }
            return Err(stop);
        }};
    }
    // The caller's outcome is decided; queue frames take the rest.
    macro_rules! finish_caller {
        ($label:lifetime, $outcome:expr) => {{
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
        let selected = current.as_ref().unwrap_or(&wave.caller_ready);
        if !queue.active() {
            // A restored output the caller's own chain consumed takes the
            // caller's mode when no demand of it has advanced.
            selected.completion().advanced_in(commit_authority);
        }
        admission
            .charge_external_work(3)
            .map_err(|_| work_denial())?;
        let producer_contacts_in_this_demand = if current.is_none() {
            // The first Ready settlement reports this demand's real contact.
            // Successful discharge resets it, so later Clean certification
            // reports no new producer contact.
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
        let result = certify_required_ready(
            runtime,
            principal,
            request_scope,
            &wave,
            selected,
            current.is_none().then_some(demand.installed_entry.as_ref()),
            resolved.as_ref(),
            producer_contacts_in_this_demand,
            installation.admission(),
        );
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
                if stack.frames.is_empty() && !queue.active() {
                    // A Clean caller with no successor only resets its contact
                    // scalar. The real demand/continuation transfer is paid
                    // inside promotion after its exact successor joins.
                    admission
                        .charge_external_work(5)
                        .map_err(|_| work_denial())?;
                    if let Some(mut successor) = demand
                        .required_continuations
                        .promote_caller_successor::<Family>(
                            &runtime.output_demands,
                            &wave.caller_ready,
                            selected,
                            current_role == FrameRole::CallerSuccessor,
                            commit_authority,
                            admission,
                        )?
                    {
                        // The newly admitted typed C demand has an empty
                        // continuation owner. Move the caller's A/B custody
                        // before its predecessor demand can be destroyed.
                        successor.required_continuations = demand.required_continuations.take_all();
                        *demand = successor;
                        let successor_interest = demand
                            .interest
                            .as_ref()
                            .expect("promoted required successor retains its Interest");
                        runtime.output_demands.finish_settlement_admitted(
                            successor_interest,
                            selected,
                            admission,
                        )?;
                        demand.producer_contacts_in_this_demand = 0;
                        finish_caller!('required, WorthQueryOutputDemandAdvance::Settled(settlement))
                    }
                    if current.is_none() {
                        let caller_interest = demand
                            .interest
                            .as_ref()
                            .expect("caller Ready retains its Interest");
                        runtime.output_demands.finish_settlement_admitted(
                            caller_interest,
                            selected,
                            admission,
                        )?;
                        demand.producer_contacts_in_this_demand = 0;
                        finish_caller!('required, WorthQueryOutputDemandAdvance::Settled(settlement))
                    }
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
                        current = Some(downstream);
                        current_contacts = 0;
                        current_role = FrameRole::Reached;
                    }
                    // An empty stack returns to the retained caller Ready.
                    continue;
                }
                finish_caller!('required, WorthQueryOutputDemandAdvance::Settled(settlement))
            }
            RequiredWaveStep::Upstream(upstream) => {
                drop(slot);
                if upstream.same_record(selected, admission)?
                    || upstream.same_record(&wave.caller_ready, admission)?
                    || resolved_on_wave.contains(&upstream)
                {
                    if queue.active() {
                        hold_queue_frame!('required, None)
                    }
                    finish_caller!('required, WorthQueryOutputDemandAdvance::Pending)
                }
                if let Some(downstream) = current.take() {
                    if !stack.push(downstream, admission)? {
                        if queue.active() {
                            hold_queue_frame!('required, None)
                        }
                        finish_caller!('required, WorthQueryOutputDemandAdvance::Pending)
                    }
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
                    RequiredFreshOutcome::Refused(stop) => {
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
                        current_contacts = successor.producer_contacts();
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
                let custody = if queue.active() {
                    &mut *frame_custody
                } else {
                    &mut demand.required_continuations
                };
                match resume_held_upstream(
                    runtime,
                    principal,
                    request_scope,
                    wave.branch,
                    custody,
                    &head,
                    admission,
                ) {
                    Ok(HeldUpstream::Ready(ready)) => {
                        runtime.output_demands.clear_required_stop(&head);
                        if committed_ready(&ready, admission)? {
                            wave = reselect_required_wave(runtime, wave, admission)?;
                            resolved_on_wave.clear();
                            queue.wave_moved();
                        }
                        // Certify the same row again against the finished upstream.
                        continue 'required;
                    }
                    Ok(HeldUpstream::Unfinished) => {
                        if queue.active() {
                            hold_queue_frame!('required, None)
                        }
                        finish_caller!('required, WorthQueryOutputDemandAdvance::Pending)
                    }
                    // The Ready the superseded successor replaced answers again.
                    Ok(HeldUpstream::GaveBack) => continue 'required,
                    Err(stop) => stopped!('required, &head, stop),
                }
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

/// What the current frame is to this wave. A caller successor is a refresh
/// of the caller's Ready, or of an earlier caller successor, reached with no
/// stacked downstream outside queue work.
#[derive(Clone, Copy, PartialEq, Eq)]
enum FrameRole {
    Reached,
    Successor,
    CallerSuccessor,
}

/// A committed Ready moved the wave past its selected position.
fn committed_ready(
    ready: &SelectedReadyReadmission,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, WorthQueryOutputDemandDenial> {
    admission
        .charge_external_work(2)
        .map_err(|_| work_denial())?;
    Ok(matches!(
        &ready.completion().authority,
        crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputAuthority::Committed(_)
    ))
}
