//! The sole final World close-admission cut, borrowed by composite owners.
use super::{drain::CloseProgress, RuntimeWorldCloseDenial as Denial};
use crate::lifecycle::owner::RuntimeWorldOwnerState;
use std::sync::{Arc, Mutex};
use worth_execution::ExecutionRequest;
mod gates;
mod receipt;
pub(crate) use receipt::RecoveredCloseReceipt;
#[cfg(test)]
mod tests;

pub(crate) fn checkpoint(request: &ExecutionRequest<'_, '_>, units: u64) -> Result<(), Denial> {
    let mut stopped = None;
    let scope = request.run_native_with_checkpoint(|context| {
        stopped = Some(if context.has_accounting() {
            context.checkpoint(units)
        } else {
            context.poll()
        });
    });
    if let Some(Err(cause)) = stopped {
        return Err(Denial::Stopped(cause));
    }
    scope.map(|_| ()).map_err(Denial::Execution)
}

/// Holds the actual final admission gates without changing World state.
/// Drop leaves Open. No supplier callback or I/O belongs inside this cut.
#[must_use]
pub struct RuntimeWorldPreCloseAdmission<'owner> {
    gates: gates::CloseAdmissionGates<'owner>,
    pub(super) task: Arc<Mutex<CloseProgress>>,
}
impl RuntimeWorldPreCloseAdmission<'_> {
    /// Infallible handoff releases all gates before owner retirement runs.
    pub fn begin(mut self) {
        self.gates.close.begin_admitted(self.task);
    }
}

fn observe<D, I, E, Ctx, T>(
    state: &RuntimeWorldOwnerState<D, I, E, Ctx, T>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    drop(gates::borrow(state, request)?);
    Ok(())
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
    observe(state, request)?;
    let frontier = state.history.close_frontier(request)?;
    // Original-request funding and task allocation happen outside owner gates.
    let task = CloseProgress::prepare_task(request, &state.recovery)?;
    let gates = gates::borrow(state, request)?;
    task.try_lock()
        .map_err(|_| Denial::Busy)?
        .validate_recovered(&state.recovery, request)?;
    state.history.try_validate_close_frontier(&frontier)?;
    request.check_live().map_err(Denial::Stopped)?;
    Ok(RuntimeWorldPreCloseAdmission { gates, task })
}

impl<D, I, E, Ctx, T> crate::lifecycle::RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Observe the same admission gates without constructing a close task.
    /// Every guard is released before returning; final prepare rechecks them.
    pub fn try_preflight_close_admission(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        observe(&self.state, request)
    }
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
    pub fn try_preflight_close_admission(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        self.root.try_preflight_close_admission(request)
    }
    pub fn try_prepare_close_admission(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RuntimeWorldPreCloseAdmission<'_>, Denial> {
        self.root.try_prepare_close_admission(request)
    }
}
