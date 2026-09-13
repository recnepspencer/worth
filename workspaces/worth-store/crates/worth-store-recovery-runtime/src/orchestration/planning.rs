mod admitted_basis;
mod completion;
mod context;
mod counters;
mod denial;
mod manifest_entry_budget;
mod operation_join;
mod page_observation;
mod resolved_basis;
mod selected_source_inventory;
mod successor_candidate_observation;

use crate::entry::PhysicalRecoveryOutcome;
use crate::progression::{PlannedPhysicalRecovery, SelectedPhysicalRecovery};

pub(crate) fn plan_recovery(
    selected: SelectedPhysicalRecovery,
) -> Result<PlannedPhysicalRecovery, PhysicalRecoveryOutcome> {
    let context = context::PlanningContext::from_selected(selected);
    let (context, admitted) = admitted_basis::admit(context)?;
    let (context, resolved) = resolved_basis::resolve(context, admitted)?;
    completion::complete(context, resolved)
}
