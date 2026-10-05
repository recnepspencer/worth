use crate::{
    authority::{ExecutionResourceLease, LeaseDenial, ResourceReservation},
    report::ChargedBytes,
};

use super::{
    admission::AdmittedBatch,
    meter::{MemoryActivityGuard, PreparedMeterContext, RunLimits},
    port::BackendKind,
};

/// The exact map memory has been admitted by the live authority. No worker is
/// reserved until the prepared map is consumed at dispatch.
pub(crate) struct PreparedBatchResources {
    memory_bytes: u64,
    _memory_reservation: ResourceReservation,
    _memory_activity: MemoryActivityGuard,
    context: PreparedMeterContext,
}

impl PreparedBatchResources {
    pub(crate) fn matches_current_parent(&self) -> bool {
        self.context.matches_current_parent()
    }

    pub(crate) fn matches_dispatch(&self, memory_bytes: u64) -> bool {
        self.memory_bytes == memory_bytes
    }

    pub(crate) fn dispatch_limits(
        &self,
        lease: &ExecutionResourceLease<'_>,
        backend: BackendKind,
    ) -> RunLimits {
        self.context
            .dispatch_limits(lease, matches!(backend, BackendKind::Serial))
    }
}

pub(crate) fn prepare_batch_resources<T: ChargedBytes, R, E>(
    lease: &ExecutionResourceLease<'_>,
    batch: &AdmittedBatch<T>,
    backend: BackendKind,
) -> Result<PreparedBatchResources, LeaseDenial> {
    if !lease.accepts_current_worker_context() {
        return Err(LeaseDenial::UnrelatedNestedLease);
    }
    let memory_bytes = batch
        .execution_memory_bytes::<R, E>()
        .and_then(|bytes| {
            bytes.checked_add(RunLimits::framework_context_bytes_for_lease(
                lease,
                batch.len(),
            )?)
        })
        .ok_or(LeaseDenial::ResourceExhausted)?;
    let reservation = lease.reserve_retained_memory(memory_bytes)?;
    let limits = RunLimits::for_run(Some(lease), matches!(backend, BackendKind::Serial));
    let context = PreparedMeterContext::capture(&limits);
    let memory_activity = limits.enter_memory(memory_bytes);
    Ok(PreparedBatchResources {
        memory_bytes,
        _memory_reservation: reservation,
        _memory_activity: memory_activity,
        context,
    })
}
