use std::num::NonZeroUsize;

use crate::branch::{
    ProductBranchHistoryTraversal, ProductBranchObservation, ProductBranchReferenceSnapshot,
    RuntimeWorldBranchAdmissionDenial,
};
use crate::identity::ProductBranchIncarnation;

use super::super::RuntimeWorldOwnerRoot;

pub(super) fn select_ancestor<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    target: &ProductBranchObservation,
    source: &crate::identity::CompositeCommitIdentity,
    maximum: Option<NonZeroUsize>,
) -> Result<
    crate::branch::ProductBranchAncestorSelection,
    crate::branch::ProductBranchAncestorSelectionDenial,
>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    use crate::branch::ProductBranchAncestorSelectionDenial as Denial;
    require_available_occurrence(owner, target.lifecycle_incarnation()).map_err(Denial::Branch)?;
    let _operation = owner
        .reserve_creation_operation()
        .map_err(|()| Denial::Branch(RuntimeWorldBranchAdmissionDenial::OwnerUnavailable))?;
    owner
        .state
        .branches
        .branch_cell_by_lifecycle(target.lifecycle_incarnation())
        .ok_or(Denial::Branch(
            RuntimeWorldBranchAdmissionDenial::RetiredBranch,
        ))?;
    let ancestor = owner
        .state
        .history
        .select_retained_ancestor(execution, target.selected_commit(), source, maximum)
        .map_err(Denial::History)?;
    Ok(crate::branch::ProductBranchAncestorSelection::owner_issued(
        target.clone(),
        ancestor,
    ))
}

pub(super) fn trace<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    occurrence: ProductBranchIncarnation,
    maximum: Option<NonZeroUsize>,
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
    let limit = maximum;
    let traversal = owner
        .state
        .history
        .trace_ancestry_within(
            crate::history::HistoryTraversalControl::Request {
                execution: *execution,
                maximum: limit,
            },
            snapshot.selected_commit().clone(),
            Some(branch_depth),
        )
        .map_err(history_denial)?;
    let remaining = branch_depth.saturating_sub(traversal.covered_generations());
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    previous: &ProductBranchHistoryTraversal,
    maximum: Option<NonZeroUsize>,
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
    let start = previous
        .next_parent()
        .cloned()
        .ok_or(RuntimeWorldBranchAdmissionDenial::HistoryEntryUnavailable)?;
    let traversal = owner
        .state
        .history
        .trace_ancestry_within(
            crate::history::HistoryTraversalControl::Request {
                execution: *execution,
                maximum,
            },
            start,
            Some(previous.remaining_branch_commits()),
        )
        .map_err(history_denial)?;
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

fn history_denial(
    denial: crate::history::CompositeHistoryCatalogDenial,
) -> RuntimeWorldBranchAdmissionDenial {
    match denial {
        crate::history::CompositeHistoryCatalogDenial::Cancelled => {
            RuntimeWorldBranchAdmissionDenial::CancelledBeforeEffect
        }
        crate::history::CompositeHistoryCatalogDenial::DeadlineElapsed => {
            RuntimeWorldBranchAdmissionDenial::DeadlineBeforeEffect
        }
        _ => RuntimeWorldBranchAdmissionDenial::HistoryEntryUnavailable,
    }
}
