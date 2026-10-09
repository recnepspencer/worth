mod admitted_basis;
mod completion;
mod context;
mod counters;
mod denial;
mod operation_join;
mod page_observation;
mod redo_limit;
mod released_directory;
mod resident_memory;
mod resolved_basis;
mod selected_source_inventory;
#[cfg(test)]
pub(super) mod selected_world_fixture;
mod successor_candidate_observation;

/// The walk's entry limit, read in recovery's limit counts.
pub(crate) use manifest_entry_budget::ExceededManifestEntries;
use page_observation::manifest_entry_budget;

use crate::entry::PhysicalRecoveryOutcome;
use crate::progression::{PlannedPhysicalRecovery, SelectedPhysicalRecovery};
pub(crate) use completion::blob_reclaim::ValidatedManifestResidueCleanup;

pub(crate) fn plan_recovery(
    selected: SelectedPhysicalRecovery,
) -> Result<PlannedPhysicalRecovery, PhysicalRecoveryOutcome> {
    let context = context::PlanningContext::from_selected(selected);
    let (context, admitted) = admitted_basis::admit(context)?;
    let (context, resolved) = resolved_basis::resolve(context, admitted)?;
    completion::complete(context, resolved)
}
