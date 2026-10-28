//! All cleanup backing is admitted before paired reference retirement.
use super::*;
use worth_execution::{
    ExecutionMemoryReservation, ExecutionRequest, LeaseDenial, WorkCeilingDenial,
};
use worth_runtime_world::facade::{ProductBranchObservation, RuntimeWorldRetiredReplacement};

pub(crate) struct WorthQueryPreparedReplacementCleanup {
    reservation: WorthQueryProductBranchOwnerCleanupReservation,
    runtime: WorthQueryProductRuntime,
    resume: Vec<CompositeCommitIdentity>,
    memory: Option<ExecutionMemoryReservation>,
    runtime_memory: Option<ExecutionMemoryReservation>,
}

impl WorthQueryProductRuntime {
    pub(crate) fn prepare_replacement_cleanup(
        &self,
        old: &ProductBranchObservation,
        request: ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryPreparedReplacementCleanup, WorthQueryCleanupReservationDenial> {
        let quote = std::alloc::Layout::array::<CompositeCommitIdentity>(1)
            .ok()
            .and_then(|layout| u64::try_from(layout.size()).ok())
            .ok_or(WorthQueryCleanupReservationDenial::SizeOverflow)?;
        let memory = request
            .reserve_retained_memory(quote)
            .map_err(WorthQueryCleanupReservationDenial::Request)?;
        let runtime_memory = runtime_copy_memory(self, request)?;
        let mut prepared = WorthQueryPreparedReplacementCleanup {
            reservation: self.owner_cleanup.reserve_funded_retirement(
                old.branch_identity(),
                old.lifecycle_incarnation(),
                request,
            )?,
            runtime: self.clone(),
            resume: Vec::new(),
            memory,
            runtime_memory,
        };
        prepared
            .resume
            .try_reserve_exact(1)
            .map_err(WorthQueryCleanupReservationDenial::BackingAllocation)?;
        if let Some(memory) = prepared.memory.as_mut() {
            let quote =
                std::alloc::Layout::array::<CompositeCommitIdentity>(prepared.resume.capacity())
                    .ok()
                    .and_then(|layout| u64::try_from(layout.size()).ok())
                    .ok_or(WorthQueryCleanupReservationDenial::SizeOverflow)?;
            memory.resize(quote).map_err(|cause| {
                WorthQueryCleanupReservationDenial::Request(WorkCeilingDenial::Admission(
                    LeaseDenial::MemoryExhausted(cause),
                ))
            })?;
        }
        Ok(prepared)
    }
}

impl WorthQueryPreparedReplacementCleanup {
    pub(crate) fn install(
        mut self,
        retired: RuntimeWorldRetiredReplacement,
    ) -> (
        WorthQueryProductBranchOwnerCleanup,
        Option<WorkCeilingDenial>,
        Option<ExecutionMemoryReservation>,
    ) {
        let (head, boundary, work, stop) = retired.into_cleanup_parts();
        let history = boundary.map(|_| {
            self.resume.push(head);
            WorthQueryProductBranchHistoryCleanup {
                resume: self.resume,
                _memory: self.memory,
            }
        });
        let record = WorthQueryProductBranchOwnerCleanupRecord {
            history,
            pending: PendingOwnerWork::Funded(work),
            retired_component_count: 0,
        };
        let id = self.reservation.install(record);
        (
            WorthQueryProductBranchOwnerCleanup::new(self.runtime, id),
            stop,
            self.runtime_memory,
        )
    }
}

fn runtime_copy_memory(
    runtime: &WorthQueryProductRuntime,
    request: ExecutionRequest<'_, '_>,
) -> Result<Option<ExecutionMemoryReservation>, WorthQueryCleanupReservationDenial> {
    // Share original Native/World grants; pay only backing actually copied by
    // this new cleanup-handle Runtime clone before invoking Clone.
    let bytes = runtime.source.cloned_backing_bytes();
    #[cfg(test)]
    let bytes = bytes
        .checked_add(runtime.default_branch.name().as_str().len())
        .ok_or(WorthQueryCleanupReservationDenial::SizeOverflow)?;
    let bytes =
        u64::try_from(bytes).map_err(|_| WorthQueryCleanupReservationDenial::SizeOverflow)?;
    if bytes == 0 {
        return Ok(None);
    }
    request
        .run_native_with_checkpoint(|context| {
            if context.has_accounting() {
                context.checkpoint(bytes)
            } else {
                context.poll()
            }
        })
        .map_err(WorthQueryCleanupReservationDenial::Request)?
        .0
        .map_err(|stop| {
            WorthQueryCleanupReservationDenial::Request(WorkCeilingDenial::Stopped(stop))
        })?;
    request
        .reserve_retained_memory(bytes)
        .map_err(WorthQueryCleanupReservationDenial::Request)
}
