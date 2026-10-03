//! One wave's required chain: the caller's own Ready first, then the dirty
//! records popped from the shared queue as frames.

use super::super::super::RequiredFreshOutcome;
use super::super::{
    WorthQueryAdmittedOutputDemand, WorthQueryOutputDemandAdvance, WorthQueryProducerOutputFamily,
};
use super::queued::RequiredQueueFrames;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::{
    registry::InstalledProducerEdition, WorthQueryProducerCommitAuthority,
};

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
    installed_edition: &InstalledProducerEdition,
    mut wave: RequiredWaveSelection<'runtime, Schema>,
    queue: &mut RequiredQueueFrames,
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
    // Whether the current frame is the successor this wave just installed.
    let mut current_is_successor = false;
    let mut last_resolved: Option<(
        SelectedReadyReadmission,
        std::sync::Arc<WorthQueryOutputDemandSettlement>,
    )> = None;
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
    // The caller's outcome is decided; queue frames take the rest.
    macro_rules! finish_caller {
        ($label:lifetime, $outcome:expr) => {{
            queue.finish_caller($outcome);
            stack.frames.clear();
            current = None;
            last_resolved = None;
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
            current_is_successor = false;
        }
        // One fixed scheduling step borrows the selected pin. Actual stack
        // writes and continuation installation are charged by their owners.
        admission
            .charge_external_work(6)
            .map_err(|_| work_denial())?;
        let selected = current.as_ref().unwrap_or(&wave.caller_ready);
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
        // Reserve caller custody before dispatch, even if it returns Current.
        let slot = demand
            .required_continuations
            .prepare_slot(&runtime.output_demands, admission)?;
        let installation_work = slot.installation_work();
        let mut installation = admission
            .reserve_external_work(installation_work)
            .map_err(admission_denial)?;
        let resolved = last_resolved.as_ref().and_then(|(ready, settled)| {
            slot.last()
                .map(|progress| ResolvedRequiredPredecessor::from_current(progress, ready, settled))
        });
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
            Err(stop) if queue.active() => {
                drop(slot);
                hold_queue_frame!('required, Some(stop))
            }
            Err(stop) => return Err(stop),
        };
        match result {
            RequiredWaveStep::Current(settlement) => {
                drop(slot);
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
                            &demand.selected.identity,
                            commit_authority,
                            installed_edition,
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
                    if stack.frames.is_empty() && queue.active() {
                        // The queued record itself is Current on this wave.
                        queue.discharge_frame(admission)?;
                        // A refreshed successor stays the resolved predecessor for
                        // later frames: their consumed edges still name the old
                        // identity, and each match is proven exactly. Any other
                        // discharged chain resolves nothing for the next one.
                        last_resolved = current_is_successor.then_some((resolved, settlement));
                        continue;
                    }
                    last_resolved = Some((resolved, settlement));
                    if let Some(downstream) = stack.pop() {
                        current = Some(downstream);
                        current_contacts = 0;
                        current_is_successor = false;
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
                    || last_resolved.as_ref().is_some_and(|(prior, _)| {
                        prior.completion().same_cell(upstream.completion())
                    })
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
                current_is_successor = false;
            }
            RequiredWaveStep::Fresh(progress) => {
                match slot.install(progress) {
                    RequiredFreshOutcome::Advanced(_) => {}
                    RequiredFreshOutcome::Refused(stop) if queue.active() => {
                        hold_queue_frame!('required, Some(stop))
                    }
                    RequiredFreshOutcome::Refused(stop) => return Err(stop),
                }
                loop {
                    // Rejoin the actual successor after each real
                    // Published/Delivered stage. A deferred stage leaves its
                    // checkpoint installed and returns Pending without
                    // spinning or reexecuting.
                    admission
                        .charge_external_work(2)
                        .map_err(|_| work_denial())?;
                    let successor = demand
                        .required_continuations
                        .last()
                        .expect("the prepared slot installed one successor");
                    if let Some(ready) = runtime
                        .output_demands
                        .interest_ready_readmission(successor.interest(), admission)?
                    {
                        admission
                            .charge_external_work(2)
                            .map_err(|_| work_denial())?;
                        let committed = matches!(
                            &ready.completion().authority,
                            crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputAuthority::Committed(_)
                        );
                        current_contacts = successor.producer_contacts();
                        if committed {
                            wave = reselect_required_wave(runtime, wave, admission)?;
                            last_resolved = None;
                            queue.wave_moved();
                        }
                        current = Some(ready);
                        current_is_successor = true;
                        continue 'required;
                    }
                    admission
                        .charge_external_work(1)
                        .map_err(|_| work_denial())?;
                    let progressed = match demand
                        .required_continuations
                        .last_mut()
                        .expect("the prepared slot installed one successor")
                        .advance_checkpoint(runtime, request_scope, admission)
                    {
                        Ok(progressed) => progressed,
                        Err(stop) if queue.active() => hold_queue_frame!('required, Some(stop)),
                        Err(stop) => return Err(stop),
                    };
                    if !progressed {
                        if queue.active() {
                            hold_queue_frame!('required, None)
                        }
                        finish_caller!('required, WorthQueryOutputDemandAdvance::Pending)
                    }
                }
            }
            RequiredWaveStep::Pending | RequiredWaveStep::NeedsDisclosure => {
                if queue.active() {
                    hold_queue_frame!('required, None)
                }
                finish_caller!('required, WorthQueryOutputDemandAdvance::Pending)
            }
        }
    }
}
