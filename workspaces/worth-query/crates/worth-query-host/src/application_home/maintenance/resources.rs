//! One Host ledger admits its joined upkeep worker and retained checkpoint data.
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
use worth_store_application::{
    ApplicationHomeCheckpointDenial as Denial, ApplicationHomeCheckpointLimits,
    ApplicationHomeCheckpointOperation,
};

const WORKER_STACK_BYTES: usize = 2 << 20;
// Native stack allocation/guard rounding fits one supported 64KiB OS granule.
const WORKER_STACK_ROUNDING_BYTES: usize = 64 << 10;

#[derive(Debug)]
pub(super) struct HomeMaintenanceResources {
    _worker_backing: ExecutionMemoryReservation,
    root: ExecutionResourceLease<'static>,
    cancellation: CancellationSource,
    timeout: std::time::Duration,
    active: AtomicBool,
}

impl HomeMaintenanceResources {
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
        let task = std::alloc::Layout::new::<(&Self, &ApplicationHomeCheckpointOperation)>();
        let handle = std::alloc::Layout::new::<std::thread::ScopedJoinHandle<'static, ()>>();
        let backing = WORKER_STACK_BYTES
            .checked_add(WORKER_STACK_ROUNDING_BYTES)
            .and_then(|bytes| bytes.checked_add(task.size()))
            .and_then(|bytes| bytes.checked_add(handle.size()))
            .ok_or(Denial::SizeLimit)?;
        // This is the Host's own root ledger, admitted before service. It is
        // never reserved against a threshold-crossing application's request.
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

    // Sanctioned category: Home automatic checkpoint isolation. Source item:
    // HomeMaintenanceResources::advance. Exactly one joined worker, never queued.
    #[expect(
        clippy::disallowed_methods,
        reason = "Home checkpoint isolation: one joined Host-funded worker separates upkeep from the caller meter"
    )]
    pub(super) fn advance(&self, operation: &ApplicationHomeCheckpointOperation) {
        if self
            .active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }
        let _active = ActiveWorker(&self.active);
        // A fresh request on this calling thread would inherit its ambient
        // meter. This joined framework worker has its own actual Host lineage.
        std::thread::scope(|scope| {
            if let Ok(worker) = std::thread::Builder::new()
                .stack_size(WORKER_STACK_BYTES)
                .spawn_scoped(scope, || self.run(operation))
                && let Err(payload) = worker.join()
            {
                std::panic::resume_unwind(payload);
            }
        });
    }

    fn run(&self, operation: &ApplicationHomeCheckpointOperation) {
        // Child construction, allocation and domain cleanup all belong to this
        // isolated execution boundary. Return unit through its scope carriers.
        let _ = ExecutionRequest::leased(&self.root).run_native(|_| {
            let Some(deadline) = Instant::now().checked_add(self.timeout) else {
                return;
            };
            let child = self
                .root
                .controlled_child(self.cancellation.token(), Some(deadline));
            let _ = operation.after_advancement(&ExecutionRequest::leased(&child));
        });
    }
}

impl Drop for HomeMaintenanceResources {
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
