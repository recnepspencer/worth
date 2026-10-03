//! Caller-owned required-wave progression before ordinary Ready acceptance.

use super::super::super::RequiredFreshOutcome;
use super::super::{
    WorthQueryAdmittedOutputDemand, WorthQueryOutputDemandAdvance, WorthQueryProducerOutputFamily,
};
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::{
    registry::InstalledProducerEdition, WorthQueryProducerCommitAuthority,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::SourceSettlementCurrentness;

/// Check actual selected required work before the caller's ordinary Ready
/// result can be accepted. The caller owns every newly minted successor.
pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) fn advance_required_before_caller<
    Schema,
    Family,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request_scope: &WorthQueryRequestScope,
    branch: WorthQueryProductBranch,
    commit_authority: &WorthQueryProducerCommitAuthority,
    installed_edition: &InstalledProducerEdition,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<WorthQueryOutputDemandAdvance>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    admission
        .charge_external_work(4)
        .map_err(|_| work_denial())?;
    let interest = demand
        .interest
        .as_ref()
        .expect("caller admission checked its live Interest");
    let Some(mut wave) = select_required_wave(runtime, interest, branch, admission)? else {
        return Ok(None);
    };
    // An Idle Ready is not itself a dirty required target. Genesis and full
    // verification cases retain the established output verifier. An authentic
    // Clean recorded row uses the selected Current proof without a source read;
    // only Dirty/Pending marks can extend the dependency closure.
    preclaim_required_settlement_arguments(admission)?;
    let candidate = runtime
        .primary_provider
        .graph
        .output_lineage
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .resolve_required_settlement(
            runtime.runtime.authority_identity().as_u64(),
            &runtime.installed_schema.binding_identity(),
            wave.caller_ready.completion(),
            admission,
        );
    let candidate = match candidate {
        Ok(candidate) => candidate,
        Err(FullVerificationReason::MarkingAdmissionDenied(stop)) => {
            return Err(admission_denial(stop));
        }
        Err(FullVerificationReason::ForeignSource) => {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "required caller accepted authority belongs to another source",
            ));
        }
        Err(_) => None,
    };
    let Some(candidate) = candidate else {
        return Ok(None);
    };
    admission
        .charge_external_work(2)
        .map_err(|_| work_denial())?;
    let posture = runtime
        .primary_provider
        .graph
        .source_owner
        .invalidation_owner
        .currentness(&wave.positioned, candidate.recorded_identity(), admission)
        .map_err(admission_denial)?;
    if matches!(
        posture,
        SourceSettlementCurrentness::FullVerificationRequired(_)
    ) {
        return Ok(None);
    }
    // Only an actual consumed-output edge can extend this closure. Initial
    // downstream B/C rows are irrelevant while their producer A is settling.
    let mut stack = RequiredWaveStack::new();
    let mut current: Option<SelectedReadyReadmission> = None;
    let mut current_contacts = 0usize;
    let mut last_resolved: Option<(
        SelectedReadyReadmission,
        std::sync::Arc<WorthQueryOutputDemandSettlement>,
    )> = None;
    'required: loop {
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
        let result = result?;
        match result {
            RequiredWaveStep::Current(settlement) => {
                drop(slot);
                if stack.frames.is_empty() {
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
                        return Ok(Some(WorthQueryOutputDemandAdvance::Settled(settlement)));
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
                        return Ok(Some(WorthQueryOutputDemandAdvance::Settled(settlement)));
                    }
                }
                if current.is_some() {
                    // Admit the reached frame transfer before moving its pin.
                    admission
                        .charge_external_work(2)
                        .map_err(|_| work_denial())?;
                }
                if let Some(resolved) = current.take() {
                    last_resolved = Some((resolved, settlement));
                    if let Some(downstream) = stack.pop() {
                        current = Some(downstream);
                        current_contacts = 0;
                    }
                    // An empty stack returns to the retained caller Ready.
                    continue;
                }
                return Ok(Some(WorthQueryOutputDemandAdvance::Settled(settlement)));
            }
            RequiredWaveStep::Upstream(upstream) => {
                drop(slot);
                if upstream.same_record(selected, admission)?
                    || upstream.same_record(&wave.caller_ready, admission)?
                    || last_resolved.as_ref().is_some_and(|(prior, _)| {
                        prior.completion().same_cell(upstream.completion())
                    })
                {
                    return Ok(Some(WorthQueryOutputDemandAdvance::Pending));
                }
                if let Some(downstream) = current.take() {
                    if !stack.push(downstream, admission)? {
                        return Ok(Some(WorthQueryOutputDemandAdvance::Pending));
                    }
                }
                current = Some(upstream);
                current_contacts = 0;
            }
            RequiredWaveStep::Fresh(progress) => {
                let outcome = slot.install(progress);
                return match outcome {
                    RequiredFreshOutcome::Advanced(_) => {
                        loop {
                            // Rejoin the actual successor after each real
                            // Published/Delivered stage. A deferred stage
                            // leaves its checkpoint installed and returns
                            // Pending without spinning or reexecuting.
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
                                }
                                current = Some(ready);
                                continue 'required;
                            }
                            admission
                                .charge_external_work(1)
                                .map_err(|_| work_denial())?;
                            let progressed = demand
                                .required_continuations
                                .last_mut()
                                .expect("the prepared slot installed one successor")
                                .advance_checkpoint(runtime, request_scope, admission)?;
                            if !progressed {
                                return Ok(Some(WorthQueryOutputDemandAdvance::Pending));
                            }
                        }
                    }
                    RequiredFreshOutcome::Refused(denial) => Err(denial),
                };
            }
            RequiredWaveStep::Pending | RequiredWaveStep::NeedsDisclosure => {
                return Ok(Some(WorthQueryOutputDemandAdvance::Pending));
            }
        }
    }
}
