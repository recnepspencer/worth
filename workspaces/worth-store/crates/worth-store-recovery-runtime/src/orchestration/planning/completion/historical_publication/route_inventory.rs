//! Complete addressed routing inventory for a historical selected root.
//! A targeted record lookup cannot prove the absence of external edges.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{CurrentPhysicalRecordPlacement, PersistedRecordIdentity};

use super::{observe_charged, HistoricalFailure};
use crate::orchestration::planning::{
    context::PlanningContext, resolved_basis::ResolvedPlanningBasis,
    selected_source_inventory::observe_routes_with_budget,
};

/// The complete, record-sorted route inventory of one exact root frame. Only
/// `observe_all_routes_of_root` mints it, after the addressed root matched
/// the frame digest its caller's custody names, so custody of one root can
/// never be handed the inventory of another root.
pub(in crate::orchestration::planning::completion) struct RootRouteInventory {
    routes: Vec<CurrentPhysicalRecordPlacement>,
}

impl RootRouteInventory {
    pub(in crate::orchestration::planning::completion) fn lacks(
        &self,
        record: PersistedRecordIdentity,
    ) -> bool {
        self.routes
            .binary_search_by_key(&record, |route| route.record())
            .is_err()
    }

    #[cfg(test)]
    pub(in crate::orchestration::planning::completion) fn for_test(
        routes: Vec<CurrentPhysicalRecordPlacement>,
    ) -> Self {
        Self { routes }
    }
}

pub(in crate::orchestration::planning::completion) fn observe_all_routes(
    context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    generation: u64,
    anchor: PersistedRecordIdentity,
) -> Result<
    (PlanningContext, Vec<CurrentPhysicalRecordPlacement>),
    crate::entry::PhysicalRecoveryOutcome,
> {
    observe_routes(context, basis, generation, anchor, None)
}

/// The inventory of the addressed root at `generation`, admitted only when
/// that root is the exact frame the caller's custody names.
pub(in crate::orchestration::planning::completion) fn observe_all_routes_of_root(
    context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    generation: u64,
    anchor: PersistedRecordIdentity,
    root_frame_sha256: [u8; 32],
) -> Result<(PlanningContext, RootRouteInventory), crate::entry::PhysicalRecoveryOutcome> {
    let (context, routes) =
        observe_routes(context, basis, generation, anchor, Some(root_frame_sha256))?;
    Ok((context, RootRouteInventory { routes }))
}

fn observe_routes(
    context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    generation: u64,
    anchor: PersistedRecordIdentity,
    root_frame_sha256: Option<[u8; 32]>,
) -> Result<
    (PlanningContext, Vec<CurrentPhysicalRecordPlacement>),
    crate::entry::PhysicalRecoveryOutcome,
> {
    let format = context.authority.record_format;
    // The root's own entry, charged before the root was read, pays for
    // every routing block of its inventory.
    observe_charged(
        context,
        basis,
        generation,
        anchor,
        |discovery, root, anchor_route, charge, budget, trace, scratch| {
            if anchor_route.is_none()
                || root_frame_sha256.is_some_and(|expected| {
                    <[u8; 32]>::from(Sha256::digest(root.encode(format))) != expected
                })
            {
                return Err(HistoricalFailure::Invalid);
            }
            let entries =
                observe_routes_with_budget(discovery, root, format, charge, budget, trace)?;
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
