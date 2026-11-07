//! Explicit, scoped entrance to reconstructive work; no ordinary fallback.
use super::{ChangeWork, MaintenanceRefusal};

/// A sealed cost-lane capability. It grants no record, schema or business
/// authority. SDK cold methods require this explicit trust-boundary scope.
pub struct ReconstructionLane<'scope> {
    _scope: std::marker::PhantomData<&'scope mut ()>,
}
pub struct ReconstructionWork<'borrow, 'scope> {
    work: &'borrow mut ChangeWork<'scope>,
}
impl ReconstructionWork<'_, '_> {
    pub fn prepare(&mut self, units: u64, bytes: u64) -> Result<(), MaintenanceRefusal> {
        self.work.prepare(units, bytes)
    }
}
impl ReconstructionLane<'_> {
    /// Explicit recovery verb. Cost: the recovered structure's named cold work;
    /// the original request still admits every traversal and allocation.
    pub fn recover<T, E: From<MaintenanceRefusal>>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        operation: impl FnOnce(&ReconstructionLane<'_>, &mut ReconstructionWork<'_, '_>) -> Result<T, E>,
    ) -> Result<T, E> {
        ChangeWork::run(execution, |work| {
            let lane = Self {
                _scope: std::marker::PhantomData,
            };
            operation(&lane, &mut ReconstructionWork { work })
        })
    }
    #[cfg(test)]
    pub(in crate::storage) fn oracle() -> Self {
        Self {
            _scope: std::marker::PhantomData,
        }
    }
}
