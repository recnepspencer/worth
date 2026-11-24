//! Fund and bind the single affinity occurrence of this exact native cell.
use super::{Arc, CompanionBranchCellCore, ManagedSourceCellAffinity};
use crate::mvcc::publication::companion::preflight_stop::CompanionPreflightStop;

impl CompanionBranchCellCore {
    pub(crate) fn retain_source_affinity(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<ManagedSourceCellAffinity>, CompanionPreflightStop> {
        if let Some(affinity) = self.source_affinity.get() {
            return Ok(Arc::clone(affinity));
        }
        let bytes = size_of::<ManagedSourceCellAffinity>()
            .checked_add(2 * size_of::<usize>())
            .and_then(|bytes| bytes.checked_add(self.branch_id.0.len()))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(|cause| CompanionPreflightStop::Execution(cause.into()))?;
        let affinity = Arc::new(ManagedSourceCellAffinity {
            runtime: self.runtime_instance_id,
            branch: self.branch_id.clone(),
            generation: self.registration_generation,
            _memory: memory,
            _fork_memory: self._fork_memory.clone(),
        });
        // All competitors derive the same immutable metadata from this exact
        // cell. Only the installed Arc is shared as its retained identity.
        let _ = self.source_affinity.set(affinity);
        Ok(Arc::clone(
            self.source_affinity.get().expect("installed cell affinity"),
        ))
    }
}
