//! Closed court custody over the production candidate backing owner.
use super::{CommitPhaseBacking, TransactionCommitError};
use worth_execution::ExecutionRequest;

pub struct RelationalCommitPhaseBackingForTest<T> {
    backing: CommitPhaseBacking<T>,
}
impl<T> RelationalCommitPhaseBackingForTest<T> {
    pub fn new(
        request: &ExecutionRequest<'_, '_>,
        value: T,
    ) -> Result<Self, TransactionCommitError> {
        Ok(Self {
            backing: CommitPhaseBacking::new(request, value)?,
        })
    }
    pub fn into_inner(self) -> T {
        self.backing.into_inner()
    }
}
