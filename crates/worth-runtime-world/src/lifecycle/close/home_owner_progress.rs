//! Original root/history cursor and its one prepaid pending retirement.
use super::home_owner::RuntimeWorldHomeRetirementDenial as Denial;
use super::{
    checkpoint_close_step, RuntimeWorldCloseDenial as CloseDenial, RuntimeWorldCloseState,
};
use crate::lifecycle::owner::RuntimeWorldOwnerState;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::RelationalBranchBasisRetirementDenial as NativeDenial, runtime::RelationalRuntime,
};
#[derive(Debug, Default)]
pub(super) struct HomeOwnerRetirementProgress {
    root_started: bool,
    root: Option<crate::history::CompositeCommitRetirement>,
    after: Option<crate::identity::CompositeCommitIdentity>,
    pending: Option<crate::history::HistoryHomeRetirement>,
    pending_complete: bool,
    complete: bool,
    retired: Vec<crate::history::HistoryHomeRetirement>,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl HomeOwnerRetirementProgress {
    pub(super) fn prepare(
        request: &ExecutionRequest<'_, '_>,
        history: &crate::history::CompositeHistoryCatalog,
    ) -> Result<Self, CloseDenial> {
        let count = history.try_home_retirement_count()?;
        let bytes = count
            .checked_mul(std::mem::size_of::<crate::history::HistoryHomeRetirement>())
            .ok_or(CloseDenial::ReportAllocationFailed)?;
        let funding = request
            .reserve_retained_memory(
                u64::try_from(bytes).map_err(|_| CloseDenial::ReportAllocationFailed)?,
            )
            .map_err(CloseDenial::Execution)?;
        let mut retired = Vec::new();
        retired
            .try_reserve_exact(count)
            .map_err(|_| CloseDenial::ReportAllocationFailed)?;
        if retired.capacity() != count {
            return Err(CloseDenial::ReportAllocationFailed);
        }
        Ok(Self {
            retired,
            _funding: funding,
            ..Default::default()
        })
    }
    pub(super) fn validate(
        &self,
        history: &crate::history::CompositeHistoryCatalog,
    ) -> Result<(), CloseDenial> {
        if history.try_home_retirement_count()? > self.retired.capacity() {
            return Err(CloseDenial::PublicationFrontierChanged);
        }
        Ok(())
    }
}
pub(super) fn advance<D, I, E, Ctx, T>(
    state: &RuntimeWorldOwnerState<D, I, E, Ctx, T>,
    request: &ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
) -> Result<bool, Denial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let close = state
        .close
        .try_lock()
        .map_err(|_| Denial::Close(CloseDenial::Busy))?;
    if close.state() != RuntimeWorldCloseState::Closed {
        return Err(Denial::Close(CloseDenial::CloseNotCompleted));
    }
    let task = close.retirement_task().map_err(Denial::Close)?;
    drop(close);
    // The progress guard spans only its own try-only metadata and nonwaiting
    // owner steps. No catalog/branch/lifecycle lock spans a Native operation.
    let mut task = task
        .try_lock()
        .map_err(|_| Denial::Close(CloseDenial::Busy))?;
    let progress = &mut task.home;
    if progress.complete {
        return Ok(true);
    }
    checkpoint_close_step(request, 1).map_err(Denial::Close)?;
    if !progress.root_started {
        progress.root = state
            .branches
            .try_begin_root_home_retirement(request)
            .map_err(Denial::Close)?;
        progress.root_started = true;
        return Ok(false);
    }
    if let Some(root) = &mut progress.root {
        if !root.advance(request, native).map_err(|cause| {
            Denial::Close(match cause {
                NativeDenial::Busy => CloseDenial::Busy,
                NativeDenial::Execution(cause) => CloseDenial::Execution(cause),
                NativeDenial::ForeignOwner => CloseDenial::CloseProgressInvariant,
            })
        })? {
            return Ok(false);
        }
    }
    if progress.pending.is_none() {
        progress.pending = state
            .history
            .try_begin_next_home_retirement(progress.after.as_ref())
            .map_err(Denial::Close)?;
        progress.pending_complete = false;
        if progress.pending.is_none() {
            if let Some(binding) = &state.durable {
                binding
                    .retire_medium_for_home(request)
                    .map_err(Denial::Medium)?;
            }
            progress.complete = true;
            return Ok(true);
        }
    }
    if !progress.pending_complete {
        progress.pending_complete = progress
            .pending
            .as_mut()
            .expect("same pending original")
            .advance(request, native)
            .map_err(Denial::Close)?;
        if !progress.pending_complete {
            return Ok(false);
        }
    }
    if progress.retired.len() == progress.retired.capacity() {
        return Err(Denial::Close(CloseDenial::CloseProgressInvariant));
    }
    let pending = progress
        .pending
        .take()
        .expect("completed actual history row");
    progress.after = Some(pending.identity().clone());
    progress.retired.push(pending);
    Ok(false)
}
