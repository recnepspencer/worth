use std::num::NonZeroUsize;

use crate::branch::{
    ProductBranchHistoryTraversal, ProductBranchObservation, ProductBranchReferenceSnapshot,
    RuntimeWorldBranchAdmissionDenial,
};
use crate::identity::ProductBranchIncarnation;

use super::super::RuntimeWorldOwnerRoot;

pub(super) fn trace<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    occurrence: ProductBranchIncarnation,
    maximum: NonZeroUsize,
) -> Result<ProductBranchHistoryTraversal, RuntimeWorldBranchAdmissionDenial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    require_available_occurrence(owner, occurrence)?;
    let _operation = owner
        .reserve_creation_operation()
        .map_err(|()| RuntimeWorldBranchAdmissionDenial::OwnerUnavailable)?;
    let cell = owner
        .state
        .branches
        .branch_cell_by_lifecycle(occurrence)
        .ok_or(RuntimeWorldBranchAdmissionDenial::RetiredBranch)?;
    let snapshot = cell.atomic_snapshot();
    let branch_depth = usize::try_from(snapshot.reference_generation().get())
        .unwrap_or(usize::MAX)
        .saturating_add(1);
    let limit = NonZeroUsize::new(maximum.get().min(branch_depth))
        .expect("a nonzero request and live branch have a nonzero history limit");
    let traversal = owner
        .state
        .history
        .trace_ancestry(snapshot.selected_commit().clone(), limit)
        .map_err(|_| RuntimeWorldBranchAdmissionDenial::HistoryEntryUnavailable)?;
    let remaining = branch_depth.saturating_sub(traversal.visited_count());
    Ok(ProductBranchHistoryTraversal::owner_issued(
        snapshot.branch_identity().clone(),
        occurrence,
        snapshot.reference_generation(),
        traversal,
        remaining,
    ))
}

pub(super) fn continue_trace<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    previous: &ProductBranchHistoryTraversal,
    maximum: NonZeroUsize,
) -> Result<ProductBranchHistoryTraversal, RuntimeWorldBranchAdmissionDenial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    require_available_occurrence(owner, previous.occurrence())?;
    let _operation = owner
        .reserve_creation_operation()
        .map_err(|()| RuntimeWorldBranchAdmissionDenial::OwnerUnavailable)?;
    owner
        .state
        .branches
        .branch_cell_by_lifecycle(previous.occurrence())
        .ok_or(RuntimeWorldBranchAdmissionDenial::RetiredBranch)?;
    let limit = previous
        .continuation_limit(maximum)
        .ok_or(RuntimeWorldBranchAdmissionDenial::HistoryEntryUnavailable)?;
    let start = previous
        .next_parent()
        .cloned()
        .ok_or(RuntimeWorldBranchAdmissionDenial::HistoryEntryUnavailable)?;
    let traversal = owner
        .state
        .history
        .trace_ancestry(start, limit)
        .map_err(|_| RuntimeWorldBranchAdmissionDenial::HistoryEntryUnavailable)?;
    previous
        .continued(traversal)
        .ok_or(RuntimeWorldBranchAdmissionDenial::HistoryEntryUnavailable)
}

pub(super) fn observe_entry<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    history: &ProductBranchHistoryTraversal,
    index: usize,
) -> Result<ProductBranchObservation, RuntimeWorldBranchAdmissionDenial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    require_available_occurrence(owner, history.occurrence())?;
    let _operation = owner
        .reserve_creation_operation()
        .map_err(|()| RuntimeWorldBranchAdmissionDenial::OwnerUnavailable)?;
    owner
        .state
        .branches
        .branch_cell_by_lifecycle(history.occurrence())
        .ok_or(RuntimeWorldBranchAdmissionDenial::RetiredBranch)?;
    let (commit, generation) = history
        .selection(index)
        .ok_or(RuntimeWorldBranchAdmissionDenial::HistoryEntryUnavailable)?;
    let capacity = owner
        .state
        .retention
        .reserve_observation()
        .map_err(super::map_retention_denial)?;
    let components = owner
        .state
        .retention
        .issue_reserved_observation(commit.as_ref(), capacity)
        .map_err(super::map_retention_denial)?;
    let history_protection = owner
        .state
        .history
        .protect_explicit_commit(commit.as_ref())
        .map_err(|_| RuntimeWorldBranchAdmissionDenial::HistoryEntryUnavailable)?;
    let snapshot = ProductBranchReferenceSnapshot::owner_issued(
        owner.owner_identity(),
        history.branch().clone(),
        history.occurrence(),
        generation,
        commit,
    )
    .map_err(|_| RuntimeWorldBranchAdmissionDenial::OwnerUnavailable)?;
    ProductBranchObservation::owner_issued(snapshot, components, history_protection)
        .map_err(|_| RuntimeWorldBranchAdmissionDenial::OwnerUnavailable)
}

fn require_available_occurrence<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    occurrence: ProductBranchIncarnation,
) -> Result<(), RuntimeWorldBranchAdmissionDenial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    if occurrence.owner_identity() != owner.owner_identity() {
        return Err(RuntimeWorldBranchAdmissionDenial::ForeignOwner);
    }
    if !owner.branch_service_is_available() {
        return Err(RuntimeWorldBranchAdmissionDenial::OwnerUnavailable);
    }
    Ok(())
}
