//! Original-request close admission and retained one-head progression.
use super::{RuntimeWorldCloseDenial as Denial, RuntimeWorldCloseReport, RuntimeWorldCloseState};
use crate::lifecycle::owner::{RuntimeWorldBootstrapState, RuntimeWorldOwnerState};
use worth_execution::ExecutionRequest;

#[derive(Debug, Default)]
pub(super) struct CloseProgress {
    pending: Option<crate::branch::registry::PendingProductHeadClose>,
    released_heads: usize,
    report: Option<RuntimeWorldCloseReport>,
}

pub(super) fn close_owner<D, I, E, Ctx, T>(
    state: &RuntimeWorldOwnerState<D, I, E, Ctx, T>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<RuntimeWorldCloseReport, Denial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let phase = state.close.try_lock().map_err(|_| Denial::Busy)?.state();
    if phase == RuntimeWorldCloseState::Closed {
        return Err(Denial::AlreadyClosed);
    }
    if !state
        .owner_present
        .load(std::sync::atomic::Ordering::Acquire)
    {
        return Err(Denial::OwnerUnavailable);
    }
    if phase == RuntimeWorldCloseState::Open {
        admit(state, request)?;
    }
    let close = state.close.try_lock().map_err(|_| Denial::Busy)?;
    if close.state() == RuntimeWorldCloseState::Closed {
        return Err(Denial::AlreadyClosed);
    }
    let task = close
        .progress
        .as_ref()
        .cloned()
        .ok_or(Denial::CloseProgressInvariant)?;
    drop(close);
    // Root permanently owns this SAME task. Try-only exclusive driving never
    // lends it out, restores it, or drops its last Arc on a budgeted return.
    // The guard spans only World-local nonwaiting metadata steps below, never
    // Native/Signal/provider calls, I/O, or live capability destruction.
    let mut progress = task.try_lock().map_err(|cause| match cause {
        std::sync::TryLockError::WouldBlock => Denial::Busy,
        std::sync::TryLockError::Poisoned(_) => Denial::CloseProgressPoisoned,
    })?;
    drain_heads(state, request, &mut progress)?;
    finish_report(&state.close, &mut progress)
}

fn drain_heads<D, I, E, Ctx, T>(
    state: &RuntimeWorldOwnerState<D, I, E, Ctx, T>,
    request: &ExecutionRequest<'_, '_>,
    progress: &mut CloseProgress,
) -> Result<(), Denial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    while progress.report.is_none() {
        request.check_live().map_err(Denial::Stopped)?;
        if progress.pending.is_none() {
            progress.pending = state
                .branches
                .try_take_next_for_close()
                .map_err(|_| Denial::Busy)?;
        }
        if let Some(pending) = progress.pending.as_mut() {
            // The actual entry is already retained before either owner phase.
            request.check_live().map_err(Denial::Stopped)?;
            pending.advance(&state.history, request)?;
            progress.released_heads += 1;
            drop(progress.pending.take());
            continue;
        }
        // This minimum slice refused unresolved business custody before seal.
        // These rows are therefore empty because the actual catalog was empty,
        // not because descriptive inspection substituted for retained custody.
        progress.report = Some(RuntimeWorldCloseReport::new(
            Vec::new(),
            Vec::new(),
            super::report::RuntimeWorldCloseReleaseCounts {
                released_product_head_pins: progress.released_heads,
                ..Default::default()
            },
            state.retention.active_observation_count(),
            Vec::new(),
        ));
    }
    Ok(())
}

fn finish_report(
    contract: &std::sync::Mutex<super::RuntimeWorldCloseContract>,
    progress: &mut CloseProgress,
) -> Result<RuntimeWorldCloseReport, Denial> {
    // Once the terminal report exists, preserve it even if this metadata lock
    // is busy. A late stop does not erase already completed owner retirement.
    let mut close = contract.try_lock().map_err(|cause| match cause {
        std::sync::TryLockError::WouldBlock => Denial::Busy,
        std::sync::TryLockError::Poisoned(_) => Denial::CloseProgressPoisoned,
    })?;
    if progress.report.is_none() {
        return Err(Denial::CloseProgressInvariant);
    }
    close.finish()?;
    let report = progress.report.take().expect("original close report");
    drop(close);
    Ok(report)
}

fn admit<D, I, E, Ctx, T>(
    state: &RuntimeWorldOwnerState<D, I, E, Ctx, T>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial>
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
    let mut close = state.close.try_lock().map_err(|_| Denial::Busy)?;
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
    close.begin()?;
    close.progress = Some(task);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Clock;
    impl crate::lifecycle::RuntimeWorldClockSource for Clock {
        fn now(&self) -> crate::lifecycle::RuntimeWorldInstant {
            crate::lifecycle::RuntimeWorldInstant::from_ticks(0)
        }
    }
    fn owner() -> (
        crate::branch::reference_test_fixture::RealReferenceFixture,
        crate::lifecycle::RuntimeWorldOwnerRoot<(), (), (), (), ()>,
    ) {
        let mut fixture = crate::branch::reference_test_fixture::real_fixture(8, 8);
        let owner = crate::lifecycle::RuntimeWorldOwnerRoot::new(fixture.owner_inputs(
            crate::budget::RuntimeWorldBudgets::without_policy_limits(),
            crate::lifecycle::RuntimeWorldClock::from_source(Clock),
        ))
        .unwrap();
        (fixture, owner)
    }
    #[test]
    fn close_finish_busy_retains_same_root_task_and_original_report() {
        let (_fixture, owner) = owner();
        let cancel = worth_execution::CancellationSource::new();
        let policy =
            crate::test_execution_policy::serial_request().with_cancellation(cancel.token());
        let request = ExecutionRequest::serial(&policy);
        admit(&owner.state, &request).unwrap();
        let task = owner
            .state
            .close
            .lock()
            .unwrap()
            .progress
            .as_ref()
            .unwrap()
            .clone();
        let mut progress = task.try_lock().unwrap();
        drain_heads(&owner.state, &request, &mut progress).unwrap();
        let report = progress.report.as_ref().unwrap() as *const RuntimeWorldCloseReport;
        let close = owner.state.close.lock().unwrap();
        assert!(std::sync::Arc::ptr_eq(
            close.progress.as_ref().unwrap(),
            &task
        ));
        assert_eq!(
            finish_report(&owner.state.close, &mut progress).unwrap_err(),
            Denial::Busy
        );
        assert_eq!(progress.report.as_ref().unwrap() as *const _, report);
        drop(progress);
        // A normal Busy return from close itself has no restoration lock or
        // lost task: the lifecycle guard is deliberately still held here.
        assert_eq!(
            close_owner(&owner.state, &request).unwrap_err(),
            Denial::Busy
        );
        drop(close);
        cancel.cancel();
        let report = close_owner(&owner.state, &request).unwrap();
        assert_eq!(report.released_product_head_pins(), 0);
        assert_eq!(
            owner.lifecycle_observation(),
            crate::lifecycle::RuntimeWorldOwnerLifecycleObservation::Closed
        );
    }
    #[test]
    fn close_poison_retains_actual_task_and_never_restores_serving() {
        let (_fixture, owner) = owner();
        let policy = crate::test_execution_policy::serial_request();
        let request = ExecutionRequest::serial(&policy);
        admit(&owner.state, &request).unwrap();
        let task = owner
            .state
            .close
            .lock()
            .unwrap()
            .progress
            .as_ref()
            .unwrap()
            .clone();
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _drive = task.try_lock().unwrap();
            panic!("actual close task unwind");
        }));
        assert!(panic.is_err());
        assert_eq!(
            close_owner(&owner.state, &request).unwrap_err(),
            Denial::CloseProgressPoisoned
        );
        let close = owner.state.close.lock().unwrap();
        assert_eq!(close.state(), RuntimeWorldCloseState::Closing);
        assert!(std::sync::Arc::ptr_eq(
            close.progress.as_ref().unwrap(),
            &task
        ));
    }
}
