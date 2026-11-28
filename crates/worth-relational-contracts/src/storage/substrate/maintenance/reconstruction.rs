//! Explicit, scoped entrance to reconstructive work; no ordinary fallback.
use super::{refusal::MaintenanceRefusal, work::ChangeWork};

/// A sealed cost-lane capability. It grants no record, schema or business
/// authority. SDK cold methods require this explicit trust-boundary scope.
pub struct ReconstructionLane<'scope> {
    _scope: std::marker::PhantomData<&'scope mut ()>,
}
pub struct ReconstructionWork<'borrow, 'scope> {
    work: &'borrow mut ChangeWork<'scope>,
}
impl<'borrow, 'scope> ReconstructionWork<'borrow, 'scope> {
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
}

/// Operations used by the Relational owners.
pub trait ReconstructionWorkOwnerPort<'borrow, 'scope>: Sized {
    fn from_change_work(work: &'borrow mut ChangeWork<'scope>) -> Self;
}

impl<'borrow, 'scope> ReconstructionWorkOwnerPort<'borrow, 'scope>
    for ReconstructionWork<'borrow, 'scope>
{
    fn from_change_work(work: &'borrow mut ChangeWork<'scope>) -> Self {
        Self { work }
    }
}

impl ReconstructionLane<'_> {
    #[cfg(test)]
    pub(crate) fn oracle() -> Self {
        Self {
            _scope: std::marker::PhantomData,
        }
    }
}
