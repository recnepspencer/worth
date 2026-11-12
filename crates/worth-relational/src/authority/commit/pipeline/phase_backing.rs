//! Original-request backing keeps consumed candidate stages off deeper frames.
use crate::transactions::data::{
    CommitExecutionDenial, CommitExecutionDenialKind, TransactionCommitError,
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

pub(crate) struct CommitPhaseBacking<T> {
    values: Vec<T>,
    _funding: Option<ExecutionMemoryReservation>,
}
impl<T> CommitPhaseBacking<T> {
    pub(crate) fn new(
        request: &ExecutionRequest<'_, '_>,
        value: T,
    ) -> Result<Self, TransactionCommitError> {
        Ok(CommitPhaseReservation::prepare(request)?.fill(value))
    }
    pub(crate) fn value(&self) -> &T {
        &self.values[0]
    }
    pub(crate) fn value_mut(&mut self) -> &mut T {
        &mut self.values[0]
    }
    pub(crate) fn into_inner(mut self) -> T {
        self.values.pop().expect("one actual candidate stage")
    }
}
pub(crate) struct CommitPhaseReservation<T> {
    values: Vec<T>,
    funding: Option<ExecutionMemoryReservation>,
}
impl<T> CommitPhaseReservation<T> {
    pub(crate) fn prepare(
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, TransactionCommitError> {
        let bytes = u64::try_from(std::mem::size_of::<T>()).map_err(|_| allocation_failure())?;
        let funding = request
            .reserve_retained_memory(bytes)
            .map_err(|cause| TransactionCommitError::execution(cause.into()))?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(1)
            .map_err(|_| allocation_failure())?;
        if values.capacity() != 1 {
            return Err(allocation_failure());
        }
        Ok(CommitPhaseReservation { values, funding })
    }
    pub(crate) fn fill(mut self, value: T) -> CommitPhaseBacking<T> {
        self.values.push(value);
        CommitPhaseBacking {
            values: self.values,
            _funding: self.funding,
        }
    }
}
fn allocation_failure() -> TransactionCommitError {
    TransactionCommitError::execution(CommitExecutionDenial {
        kind: CommitExecutionDenialKind::Cause(
            crate::execution::RelationalExecutionDenialCause::AllocationFailed,
        ),
        partition_identity: None,
    })
}

mod control;
pub use control::RelationalCommitPhaseBackingForTest;
