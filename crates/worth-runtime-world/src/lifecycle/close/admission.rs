//! The sole final World close-admission cut, borrowed by composite owners.
use super::{
    drain::CloseProgress, RuntimeWorldCloseContract, RuntimeWorldCloseDenial as Denial,
    RuntimeWorldCloseState,
};
use crate::lifecycle::owner::{
    RuntimeWorldBootstrapState, RuntimeWorldOperationLedgerState, RuntimeWorldOwnerState,
};
use std::sync::{Arc, Mutex, MutexGuard};
use worth_execution::ExecutionRequest;

/// Holds the actual final admission gates without changing World state.
/// Drop leaves Open. Only `begin` seals the same owner and installs its task.
/// No supplier callbacks, I/O or other fallible work belongs inside this cut.
#[must_use]
pub struct RuntimeWorldPreCloseAdmission<'owner> {
    _bootstrap: MutexGuard<'owner, RuntimeWorldBootstrapState>,
    _operation: MutexGuard<'owner, RuntimeWorldOperationLedgerState>,
    close: MutexGuard<'owner, RuntimeWorldCloseContract>,
    pub(super) task: Arc<Mutex<CloseProgress>>,
}
impl RuntimeWorldPreCloseAdmission<'_> {
    /// Infallible handoff after every admission check. Releases all gates before
    /// the existing close driver performs retirement or component work.
    pub fn begin(mut self) {
        self.close.begin_admitted(self.task);
    }
}

pub(super) fn prepare<'owner, D, I, E, Ctx, T>(
    state: &'owner RuntimeWorldOwnerState<D, I, E, Ctx, T>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<RuntimeWorldPreCloseAdmission<'owner>, Denial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    request.check_live().map_err(Denial::Stopped)?;
    {
        let operation = state.operation.state.try_lock().map_err(|_| Denial::Busy)?;
        if operation.active != 0 || operation.recovery_active != 0 {
            return Err(Denial::InFlightCriticalSection);
        }
    }
    // Cheap actual-custody rejection precedes a history pass. Neither this
    // observation nor a frontier replaces the final same-gate recheck below.
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
    let frontier = state.history.close_frontier(request)?;
    let task = std::sync::Arc::new(std::sync::Mutex::new(CloseProgress::default()));
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
    state.history.try_validate_close_frontier(&frontier)?;
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
    Ok(RuntimeWorldPreCloseAdmission {
        _bootstrap: bootstrap,
        _operation: operation,
        close,
        task,
    })
}

impl<D, I, E, Ctx, T> crate::lifecycle::RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Borrow the final admission gates for a short owner-only composite cut.
    pub fn try_prepare_close_admission(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RuntimeWorldPreCloseAdmission<'_>, Denial> {
        prepare(&self.state, request)
    }
}
impl<D, I, E, Ctx, T> crate::lifecycle::RuntimeWorldOwner<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn try_prepare_close_admission(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RuntimeWorldPreCloseAdmission<'_>, Denial> {
        self.root.try_prepare_close_admission(request)
    }
}
