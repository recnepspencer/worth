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
    /// Constructor restricted by the exact trust-boundary caller registry.
    /// Ordinary SDK paths and downstream production adopters cannot mint one:
    /// boundary-check rejects every constructor reference outside that registry.
    pub fn enter_reconstruction<T>(operation: impl FnOnce(&ReconstructionLane<'_>) -> T) -> T {
        let lane = Self {
            _scope: std::marker::PhantomData,
        };
        operation(&lane)
    }
    /// Explicit recovery verb. Cost: the recovered structure's named cold work;
    /// the original request still admits every traversal and allocation.
    pub fn recover<T, E: From<MaintenanceRefusal>>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        operation: impl FnOnce(&ReconstructionLane<'_>, &mut ReconstructionWork<'_, '_>) -> Result<T, E>,
    ) -> Result<T, E> {
        ChangeWork::run(execution, |work| {
            operation(self, &mut ReconstructionWork { work })
        })
    }
    /// Private-logic oracles exercise the same cold engines without constructing
    /// a foreign execution runtime inside a source unit test.
    #[cfg(test)]
    pub(crate) fn inspect_oracle<T>(
        operation: impl FnOnce(&ReconstructionLane<'_>, &mut ReconstructionWork<'_, '_>) -> T,
    ) -> T {
        ChangeWork::oracle(|work| operation(&Self::oracle(), &mut ReconstructionWork { work }))
    }
    #[cfg(test)]
    pub(crate) fn oracle() -> Self {
        Self {
            _scope: std::marker::PhantomData,
        }
    }
}
