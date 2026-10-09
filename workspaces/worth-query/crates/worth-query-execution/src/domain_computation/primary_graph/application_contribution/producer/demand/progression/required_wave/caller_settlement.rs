//! Promotion and settlement preserve the caller's demand custody and contacts.
use super::super::{WorthQueryAdmittedOutputDemand, WorthQueryProducerOutputFamily};
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::{
    registry::InstalledProducerEdition, WorthQueryProducerCommitAuthority,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn settle_selected_caller<Schema, Family>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
    wave: &RequiredWaveSelection<'_, Schema>,
    selected: &SelectedReadyReadmission,
    caller_successor: bool,
    caller_is_selected: bool,
    current_contacts: usize,
    commit_authority: &WorthQueryProducerCommitAuthority,
    installed_edition: &InstalledProducerEdition,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    // A Clean caller keeps its cumulative contact count.
    // The real demand/continuation transfer is paid
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
            caller_successor,
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
        successor.producer_contacts_in_this_demand = current_contacts;
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
        // A settled chain has no unfinished successor. Its wave pins end here;
        // the Ready rows retain their own custody for their live owners.
        drop(demand.required_continuations.take_all());
        // Settlement retains all executions initiated by this demand.
        return Ok(true);
    }
    if caller_is_selected {
        let caller_interest = demand
            .interest
            .as_ref()
            .expect("caller Ready retains its Interest");
        runtime
            .output_demands
            .finish_settlement_admitted(caller_interest, selected, admission)?;
        // A settled chain has no unfinished successor. Its wave pins end here;
        // the Ready rows retain their own custody for their live owners.
        drop(demand.required_continuations.take_all());
        // Settlement retains all executions initiated by this demand.
        return Ok(true);
    }
    Ok(false)
}
