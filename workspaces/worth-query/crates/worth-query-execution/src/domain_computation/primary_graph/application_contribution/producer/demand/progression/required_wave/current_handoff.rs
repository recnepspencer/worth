//! A certified Current successor transfers custody while the caller keeps its advance mode.

use super::super::super::required_provenance::DemandProgressionProvenance;
use super::super::{WorthQueryAdmittedOutputDemand, WorthQueryProducerOutputFamily};
use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn finish_current_caller<Schema, Family>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
    anchor_ready: &SelectedReadyReadmission,
    selected: &SelectedReadyReadmission,
    continues_caller: bool,
    caller_current: bool,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    // A Clean caller without a successor only resets its contact scalar.
    // Promotion pays the actual demand, continuation and provenance moves.
    admission
        .charge_external_work(5)
        .map_err(|_| work_denial())?;
    if let Some(mut successor) = demand
        .required_continuations
        .promote_caller_successor::<Family>(
            &runtime.output_demands,
            anchor_ready,
            selected,
            continues_caller,
            admission,
        )?
    {
        // The successor completed under its own issued mode. This is a
        // Current custody handoff, so future advances retain the caller's
        // original authority, as when an open demand rejoins another refresh.
        let mut provenance = std::mem::take(&mut demand.progression_provenance);
        if let DemandProgressionProvenance::RequiredSuccessor(required) = &mut provenance {
            required.bind_successor(successor.installed_entry.edition);
        }
        successor.progression_provenance = provenance;
        successor.required_continuations = demand.required_continuations.take_all();
        *demand = successor;
    } else if !caller_current {
        return Ok(false);
    }
    let interest = demand
        .interest
        .as_ref()
        .expect("Current caller retains its Interest");
    runtime
        .output_demands
        .finish_settlement_admitted(interest, selected, admission)?;
    demand.producer_contacts_in_this_demand = 0;
    Ok(true)
}
