//! Exact retained history is admitted independently of live branch references.
use super::super::RuntimeWorldOwnerRoot;
use crate::branch::{
    PreparedProductExactHistoryPage, ProductBranchObservation, ProductExactHistoryCursor,
    ProductExactHistoryDenial as Denial, RuntimeWorldBranchAdmissionDenial,
};

pub(super) fn open<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    target: &ProductBranchObservation,
) -> Result<ProductExactHistoryCursor, Denial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    require_owner(owner, target)?;
    let _operation = owner
        .reserve_creation_operation()
        .map_err(|()| Denial::Branch(RuntimeWorldBranchAdmissionDenial::OwnerUnavailable))?;
    request_stage(execution, |checkpoint| {
        checkpoint_step(checkpoint)?;
        let position = owner
            .state
            .history
            .retain_exact_history_position(execution, target.selected_commit())
            .map_err(Denial::History)?;
        ProductExactHistoryCursor::owner_issued(execution, target.clone(), position)
    })
}

pub(super) fn prepare<'request, 'authority, D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    execution: worth_execution::ExecutionRequest<'request, 'authority>,
    cursor: &ProductExactHistoryCursor,
) -> Result<PreparedProductExactHistoryPage<'request, 'authority>, Denial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    require_owner(owner, cursor.target())?;
    let _operation = owner
        .reserve_creation_operation()
        .map_err(|()| Denial::Branch(RuntimeWorldBranchAdmissionDenial::OwnerUnavailable))?;
    let next = request_stage(execution, |checkpoint| {
        let Some(current) = cursor.position() else {
            return Ok(None);
        };
        checkpoint_step(checkpoint)?;
        owner
            .state
            .history
            .next_exact_history_position(execution, current)
            .map_err(Denial::History)
    })?;
    Ok(cursor.prepared(execution, next))
}

fn require_owner<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    target: &ProductBranchObservation,
) -> Result<(), Denial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    if target.owner_identity() != owner.owner_identity() {
        return Err(Denial::Branch(
            RuntimeWorldBranchAdmissionDenial::ForeignOwner,
        ));
    }
    if !owner.branch_service_is_available() {
        return Err(Denial::Branch(
            RuntimeWorldBranchAdmissionDenial::OwnerUnavailable,
        ));
    }
    Ok(())
}

fn checkpoint_step(
    checkpoint: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<(), Denial> {
    if checkpoint.has_accounting() {
        checkpoint.checkpoint(1)
    } else {
        checkpoint.poll()
    }
    .map_err(|stop| Denial::Request(worth_execution::WorkCeilingDenial::Stopped(stop)))
}

fn request_stage<T>(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    operation: impl FnOnce(&mut worth_execution::MapKernelContext<'_, '_>) -> Result<T, Denial>,
) -> Result<T, Denial> {
    use worth_execution::{MapKernelStop, WorkCeilingDenial};
    let mut value = None;
    let outcome = execution.run_native_with_checkpoint(|context| {
        value = Some(operation(context));
    });
    match (outcome, value) {
        (Ok(_), Some(value)) => value,
        (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
            Err(cause)
        }
        (Err(cause), _) => Err(Denial::Request(cause)),
        (Ok(_), None) => unreachable!("completed World history callback"),
    }
}
