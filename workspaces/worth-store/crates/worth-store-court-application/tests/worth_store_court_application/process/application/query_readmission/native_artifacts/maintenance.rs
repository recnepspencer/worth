//! Observational attachments retain the real Home's advancement operation.
use super::*;

#[derive(Debug)]
struct HomeMaintenance {
    operation: ApplicationHomeCheckpointOperation,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl WorthQueryHomeMaintenance for HomeMaintenance {
    fn after_advancement(&self, request: &ExecutionRequest<'_, '_>) {
        let _ = self.operation.after_advancement(request);
    }
}

pub(super) fn prepare(
    home: &DirectoryHome,
    request: &ExecutionRequest<'_, '_>,
) -> Arc<dyn WorthQueryHomeMaintenance> {
    let operation = home
        .checkpoint_operation(ApplicationHomeCheckpointLimits::default())
        .unwrap();
    let bytes = std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<HomeMaintenance>())
        .unwrap()
        .0
        .pad_to_align()
        .size();
    let memory = request.reserve_retained_memory(bytes as u64).unwrap();
    Arc::new(HomeMaintenance {
        operation,
        _memory: memory,
    })
}
