//! A closed court retains the production completion cell and original grant.
use super::{ExecutionRequest, PhaseStorage, WorthQueryApplicationCommitOutcome};

pub struct WorthQueryManagedPhaseStorageForTest<T> {
    storage: PhaseStorage<T>,
}
impl<T> WorthQueryManagedPhaseStorageForTest<T> {
    pub fn prepare(
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, WorthQueryApplicationCommitOutcome> {
        Ok(Self {
            storage: PhaseStorage::prepare(request)?,
        })
    }
    pub fn fill(self, actual: T) -> Self {
        Self {
            storage: self.storage.fill(actual),
        }
    }
    pub fn consume(self) -> T {
        self.storage.consume()
    }
}
