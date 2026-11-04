//! One Home ledger admits its joined upkeep worker and retained checkpoint data.
use crate::{
    ApplicationHomeCheckpoint as Summary, ApplicationHomeCheckpointDenial as Denial,
    ApplicationHomeCheckpointLimits, ApplicationHomeCheckpointMaintenanceDenial as Maintenance,
    ApplicationHomeCheckpointOperation,
};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
use worth_execution::{
    CancellationSource, ExecutionMemoryReservation, ExecutionRequest, ExecutionResourceLease,
    LeaseDenial, LeaseRequest, WorkCeilingDenial,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

const WORKER_STACK_BYTES: usize = 2 << 20;
// Native stack allocation/guard rounding fits one supported 64KiB OS granule.
const WORKER_STACK_ROUNDING_BYTES: usize = 64 << 10;

#[derive(Debug)]
pub(super) struct MaintenanceWorker {
    root: ExecutionResourceLease<'static>,
    cancellation: CancellationSource,
    timeout: std::time::Duration,
    active: AtomicBool,
    // Last: all payload fields retire before the allocation/stack bill.
    _worker_backing: ExecutionMemoryReservation,
}

impl MaintenanceWorker {
    pub(super) fn install(limits: ApplicationHomeCheckpointLimits) -> Result<Self, Denial> {
        let cancellation = CancellationSource::new();
        let root = ExecutionResourceLease::serial(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(
                    std::num::NonZeroUsize::MIN,
                    Some(limits.maintenance_memory_bytes().get()),
                    Some(limits.maintenance_work().get()),
                ),
            ),
            deadline: None,
            cancellation: cancellation.token(),
        })
        .map_err(|cause| Denial::Execution(WorkCeilingDenial::Admission(cause)))?;
        let task = std::alloc::Layout::new::<(
            &Self,
            &ApplicationHomeCheckpointOperation,
            &Purpose,
            &mut Option<Result<Option<Summary>, Denial>>,
        )>();
        let handle = std::alloc::Layout::new::<
            std::thread::ScopedJoinHandle<'static, Result<(), WorkCeilingDenial>>,
        >();
        let owner = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .map_err(|_| Denial::SizeLimit)?
            .0
            .pad_to_align();
        let backing = WORKER_STACK_BYTES
            .checked_add(WORKER_STACK_ROUNDING_BYTES)
            .and_then(|bytes| bytes.checked_add(task.size()))
            .and_then(|bytes| bytes.checked_add(handle.size()))
            .and_then(|bytes| bytes.checked_add(owner.size()))
            .ok_or(Denial::SizeLimit)?;
        // Host installation admits this Home ledger before service; bare
        // Store close admits it on demand. No caller request funds it.
        let memory = root.reserve_memory(backing as u64).map_err(|cause| {
            Denial::Execution(WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(
                cause,
            )))
        })?;
        Ok(Self {
            _worker_backing: memory,
            root,
            cancellation,
            timeout: limits.maintenance_timeout(),
            active: AtomicBool::new(false),
        })
    }

    pub(super) fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    pub(super) fn advance(
        &self,
        operation: &ApplicationHomeCheckpointOperation,
    ) -> Result<Option<Summary>, Denial> {
        self.execute(operation, Purpose::Advancement)
    }
    pub(super) fn capture_for_close(
        &self,
        operation: &ApplicationHomeCheckpointOperation,
    ) -> Result<Summary, Denial> {
        self.execute(operation, Purpose::Close)?
            .ok_or(Denial::Maintenance(Maintenance::Unavailable))
    }
    fn execute(
        &self,
        operation: &ApplicationHomeCheckpointOperation,
        purpose: Purpose,
    ) -> Result<Option<Summary>, Denial> {
        let mut result = Err(Denial::Maintenance(Maintenance::WorkerPanicked));
        super::panic_boundary::contain(|| result = self.execute_scoped(operation, purpose));
        result
    }
    // Sanctioned category: Home automatic checkpoint isolation. Source item:
    // MaintenanceWorker::execute_scoped. Exactly one joined worker, never queued.
    #[expect(
        clippy::disallowed_methods,
        reason = "Home checkpoint isolation: one joined Host-funded worker separates automatic advancement/close capture from caller meters"
    )]
    fn execute_scoped(
        &self,
        operation: &ApplicationHomeCheckpointOperation,
        purpose: Purpose,
    ) -> Result<Option<Summary>, Denial> {
        self.active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| Denial::Maintenance(Maintenance::Busy))?;
        let _active = ActiveWorker(&self.active);
        let mut outcome = None;
        let run = std::thread::scope(|scope| {
            let worker = std::thread::Builder::new()
                .stack_size(WORKER_STACK_BYTES)
                .spawn_scoped(scope, || self.run(operation, purpose, &mut outcome))
                .map_err(|cause| Denial::Maintenance(Maintenance::WorkerCreation(cause.kind())))?;
            match worker.join() {
                Ok(run) => Ok(run),
                Err(payload) => std::panic::resume_unwind(payload),
            }
        })?;
        match outcome {
            Some(result) => result,
            None => {
                run.map_err(Denial::Execution)?;
                Err(Denial::Maintenance(Maintenance::Unavailable))
            }
        }
    }
    fn run(
        &self,
        operation: &ApplicationHomeCheckpointOperation,
        purpose: Purpose,
        outcome: &mut Option<Result<Option<Summary>, Denial>>,
    ) -> Result<(), WorkCeilingDenial> {
        ExecutionRequest::leased(&self.root)
            .run_native(|_| {
                let Some(deadline) = Instant::now().checked_add(self.timeout) else {
                    *outcome = Some(Err(Denial::Maintenance(Maintenance::DeadlineOverflow)));
                    return;
                };
                let child = self
                    .root
                    .controlled_child(self.cancellation.token(), Some(deadline));
                let request = ExecutionRequest::leased(&child);
                *outcome = Some(match purpose {
                    Purpose::Advancement => operation.after_advancement(&request),
                    Purpose::Close => operation.checkpoint(&request).map(Some),
                });
            })
            .map(|_| ())
    }
}

impl Drop for MaintenanceWorker {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}

struct ActiveWorker<'a>(&'a AtomicBool);
impl Drop for ActiveWorker<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

#[derive(Clone, Copy)]
enum Purpose {
    Advancement,
    Close,
}
