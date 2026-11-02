//! The closed Directory Host installs the actual same-Home Store operation.
use super::runtime::DirectoryHome;
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_query_execution::facade::application_home_integration::WorthQueryHomeMaintenance;
use worth_store_application::{
    ApplicationHomeCheckpointDenial as Denial, ApplicationHomeCheckpointLimits,
    ApplicationHomeCheckpointOperation,
};

#[derive(Debug)]
struct HomeMaintenance {
    operation: ApplicationHomeCheckpointOperation,
    _memory: Option<ExecutionMemoryReservation>,
}
impl WorthQueryHomeMaintenance for HomeMaintenance {
    fn after_advancement(&self, request: &ExecutionRequest<'_, '_>) {
        // Query retains the exact business value. Actual work/refusal stays in
        // this SAME Store operation's attempt/coverage; there is no error ledger.
        let _ = self.operation.after_advancement(request);
    }
}
pub(super) fn prepare(
    home: &DirectoryHome,
    limits: ApplicationHomeCheckpointLimits,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Arc<dyn WorthQueryHomeMaintenance>, Denial> {
    let operation = home.checkpoint_operation(limits)?;
    let bytes = std::alloc::Layout::new::<(usize, usize)>()
        .extend(std::alloc::Layout::new::<HomeMaintenance>())
        .map_err(|_| Denial::SizeLimit)?
        .0
        .pad_to_align()
        .size();
    let memory = request
        .reserve_retained_memory(bytes as u64)
        .map_err(Denial::Execution)?;
    // This private Arc is only installed in Query; no public Arc/Weak door
    // exposes its backing independently of the operation and LAST grant.
    Ok(Arc::new(HomeMaintenance {
        operation,
        _memory: memory,
    }))
}
