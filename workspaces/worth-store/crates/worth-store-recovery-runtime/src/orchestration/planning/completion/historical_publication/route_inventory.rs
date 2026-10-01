//! Complete addressed routing inventory for a historical selected root.
//! A targeted record lookup cannot prove the absence of external edges.

use worth_store_physical_format::{CurrentPhysicalRecordPlacement, PersistedRecordIdentity};

use super::{observe, HistoricalFailure};
use crate::orchestration::planning::{
    context::PlanningContext, resolved_basis::ResolvedPlanningBasis,
    selected_source_inventory::observe_routes_with_budget,
};

pub(in crate::orchestration::planning::completion) fn observe_all_routes(
    context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    generation: u64,
    anchor: PersistedRecordIdentity,
) -> Result<
    (PlanningContext, Vec<CurrentPhysicalRecordPlacement>),
    crate::entry::PhysicalRecoveryOutcome,
> {
    let format = context.authority.record_format;
    observe(
        context,
        basis,
        generation,
        anchor,
        |discovery, root, anchor_route, budget, trace, scratch| {
            if anchor_route.is_none() {
                return Err(HistoricalFailure::Invalid);
            }
            let entries = observe_routes_with_budget(discovery, root, format, budget, trace)
                .map_err(|_| HistoricalFailure::Invalid)?;
            if entries
                .binary_search_by_key(&anchor, |placement| placement.record())
                .is_err()
            {
                return Err(HistoricalFailure::Invalid);
            }
            *scratch = (*scratch).max(
                (entries.len() * std::mem::size_of::<CurrentPhysicalRecordPlacement>()) as u64,
            );
            Ok(entries)
        },
    )
}
