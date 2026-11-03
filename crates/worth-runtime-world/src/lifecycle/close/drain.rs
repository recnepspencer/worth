//! Original-request close admission and retained one-head progression.
use super::{RuntimeWorldCloseDenial as Denial, RuntimeWorldCloseReport, RuntimeWorldCloseState};
use crate::lifecycle::owner::RuntimeWorldOwnerState;
use worth_execution::ExecutionRequest;

#[derive(Debug, Default)]
pub(super) struct CloseProgress {
    pending: Option<crate::branch::registry::PendingProductHeadClose>,
    released_heads: usize,
    report: Option<RuntimeWorldCloseReport>,
    recovered: Vec<super::RuntimeWorldRecoveredCustodyReport>,
    recovered_funding: Option<worth_execution::ExecutionMemoryReservation>,
    _metadata_funding: Option<worth_execution::ExecutionMemoryReservation>,
}

impl CloseProgress {
    #[cfg(test)]
    pub(super) fn prepare_task(
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<std::sync::Arc<std::sync::Mutex<Self>>, Denial> {
        Self::prepare_task_with_recovered(request, 0)
    }

    pub(super) fn prepare_task_with_recovered(
        request: &ExecutionRequest<'_, '_>,
        recovered_count: usize,
    ) -> Result<std::sync::Arc<std::sync::Mutex<Self>>, Denial> {
        let (layout, _) = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(std::alloc::Layout::new::<std::sync::Mutex<Self>>())
            .expect("fixed World close task layout fits");
        let funding = request
            .reserve_retained_memory(layout.pad_to_align().size() as u64)
            .map_err(Denial::Execution)?;
        let bytes = recovered_count
            .checked_mul(std::mem::size_of::<super::RuntimeWorldRecoveredCustodyReport>())
            .ok_or(Denial::CloseProgressInvariant)?;
        let recovered_funding = request
            .reserve_retained_memory(bytes as u64)
            .map_err(Denial::Execution)?;
        let mut recovered = Vec::new();
        recovered
            .try_reserve_exact(recovered_count)
            .map_err(|_| Denial::CloseProgressInvariant)?;
        if recovered.capacity() != recovered_count {
            return Err(Denial::CloseProgressInvariant);
        }
        Ok(std::sync::Arc::new(std::sync::Mutex::new(Self {
            pending: None,
            released_heads: 0,
            report: None,
            recovered,
            recovered_funding,
            _metadata_funding: funding,
        })))
    }
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
        state
            .recovery
            .append_close_reports(&mut progress.recovered, request)?;
        progress.report = Some(RuntimeWorldCloseReport::new(
            Vec::new(),
            std::mem::take(&mut progress.recovered),
            progress.recovered_funding.take(),
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
    super::admission::prepare(state, request)?.begin();
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
    #[test]
    fn borrowed_preclose_guard_excludes_calls_and_drop_preserves_open() {
        let (fixture, owner) = owner();
        assert!(matches!(
            owner.bootstrap_root(fixture.bootstrap_intent()),
            crate::branch::RuntimeWorldBootstrapOutcome::Performed(_)
        ));
        let policy = crate::test_execution_policy::serial_request();
        let request = ExecutionRequest::serial(&policy);
        assert!(owner.try_admit_call().is_some());
        let admission = owner.try_prepare_close_admission(&request).unwrap();
        assert!(owner.try_admit_call().is_none());
        assert!(matches!(
            close_owner(&owner.state, &request),
            Err(Denial::Busy)
        ));
        drop(admission);
        assert_eq!(
            owner.lifecycle_observation(),
            crate::lifecycle::RuntimeWorldOwnerLifecycleObservation::Open
        );
        assert!(owner.try_admit_call().is_some());
        assert!(owner.state.bootstrap.try_lock().is_ok());
        assert!(owner.state.operation.state.try_lock().is_ok());
        assert!(owner.state.close.try_lock().is_ok());
        assert!(owner.close(&request).unwrap().retained_records().is_empty());
    }
    #[test]
    fn preclose_begin_ignores_late_stop_and_retains_the_same_close_task() {
        let (_fixture, owner) = owner();
        let cancel = worth_execution::CancellationSource::new();
        let policy =
            crate::test_execution_policy::serial_request().with_cancellation(cancel.token());
        let request = ExecutionRequest::serial(&policy);
        let admission = owner.try_prepare_close_admission(&request).unwrap();
        let task = admission.task.clone();
        cancel.cancel();
        admission.begin();
        assert_eq!(
            owner.lifecycle_observation(),
            crate::lifecycle::RuntimeWorldOwnerLifecycleObservation::Closing
        );
        assert!(std::sync::Arc::ptr_eq(
            owner.state.close.lock().unwrap().progress.as_ref().unwrap(),
            &task
        ));
        assert!(matches!(owner.close(&request), Err(Denial::Stopped(_))));
        let retry = crate::test_execution_policy::serial_request();
        assert!(owner
            .close(&ExecutionRequest::serial(&retry))
            .unwrap()
            .retained_records()
            .is_empty());
    }
    #[test]
    fn active_recovery_and_request_stop_refuse_without_sealing() {
        let (fixture, owner) = owner();
        assert!(matches!(
            owner.bootstrap_root(fixture.bootstrap_intent()),
            crate::branch::RuntimeWorldBootstrapOutcome::Performed(_)
        ));
        let policy = crate::test_execution_policy::serial_request();
        let request = ExecutionRequest::serial(&policy);
        let active = owner.try_admit_call().unwrap();
        assert!(matches!(
            owner.try_prepare_close_admission(&request),
            Err(Denial::InFlightCriticalSection)
        ));
        drop(active);
        let recovery = owner
            .reserve_recovery_operation_if_open_and_bootstrapped()
            .unwrap();
        assert!(matches!(
            owner.try_prepare_close_admission(&request),
            Err(Denial::InFlightCriticalSection)
        ));
        drop(recovery);
        let cancel = worth_execution::CancellationSource::new();
        cancel.cancel();
        let stopped = policy.with_cancellation(cancel.token());
        assert!(matches!(
            owner.try_prepare_close_admission(&ExecutionRequest::serial(&stopped)),
            Err(Denial::Stopped(_))
        ));
        assert_eq!(
            owner.lifecycle_observation(),
            crate::lifecycle::RuntimeWorldOwnerLifecycleObservation::Open
        );
    }
}
