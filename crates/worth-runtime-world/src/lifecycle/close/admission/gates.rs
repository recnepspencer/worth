//! One actual admission policy shared by the early observation and final cut.
use super::Denial;
use crate::lifecycle::{
    close::{RuntimeWorldCloseContract, RuntimeWorldCloseState},
    owner::{RuntimeWorldBootstrapState, RuntimeWorldOperationLedgerState, RuntimeWorldOwnerState},
};
use std::sync::MutexGuard;
use worth_execution::ExecutionRequest;

pub(super) struct CloseAdmissionGates<'owner> {
    _bootstrap: MutexGuard<'owner, RuntimeWorldBootstrapState>,
    _operation: MutexGuard<'owner, RuntimeWorldOperationLedgerState>,
    pub(super) close: MutexGuard<'owner, RuntimeWorldCloseContract>,
}

pub(super) fn borrow<'owner, D, I, E, Ctx, T>(
    state: &'owner RuntimeWorldOwnerState<D, I, E, Ctx, T>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<CloseAdmissionGates<'owner>, Denial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    request.check_live().map_err(Denial::Stopped)?;
    let bootstrap = state.bootstrap.try_lock().map_err(|_| Denial::Busy)?;
    match &*bootstrap {
        RuntimeWorldBootstrapState::InProgress => return Err(Denial::AlreadyClosing),
        RuntimeWorldBootstrapState::Pending(_) => return Err(Denial::UnresolvedBusinessCustody),
        _ => {}
    }
    let operation = state.operation.state.try_lock().map_err(|_| Denial::Busy)?;
    if operation.active != 0 || operation.recovery_active != 0 {
        return Err(Denial::InFlightCriticalSection);
    }
    let close = state.close.try_lock().map_err(|_| Denial::Busy)?;
    if close.state() != RuntimeWorldCloseState::Open {
        return Err(Denial::AlreadyClosing);
    }
    if !state
        .owner_present
        .load(std::sync::atomic::Ordering::Acquire)
    {
        return Err(Denial::OwnerUnavailable);
    }
    if state
        .recovery
        .try_has_close_custody()
        .map_err(|_| Denial::Busy)?
        || state
            .custody
            .try_has_close_custody()
            .map_err(|_| Denial::Busy)?
    {
        return Err(Denial::UnresolvedBusinessCustody);
    }
    if let Some(binding) = &state.durable {
        binding.try_admit_close()?;
    }
    request.check_live().map_err(Denial::Stopped)?;
    Ok(CloseAdmissionGates {
        _bootstrap: bootstrap,
        _operation: operation,
        close,
    })
}
