use super::{
    custody::PhysicalCloseTaskCustody, PhysicalStoreCloseObservation, PhysicalStoreCloseOutcome,
};
use crate::physical_runtime::{RuntimeIdentity, ServingPhysicalRuntime};

/// Thread creation refused before the actual Store was taken by a worker.
#[must_use]
pub struct PhysicalStoreCloseStartRefused {
    serving: ServingPhysicalRuntime,
    error: std::io::Error,
}

impl PhysicalStoreCloseStartRefused {
    pub(super) fn new(serving: ServingPhysicalRuntime, error: std::io::Error) -> Self {
        Self { serving, error }
    }

    pub fn error(&self) -> &std::io::Error {
        &self.error
    }

    pub fn into_parts(self) -> (ServingPhysicalRuntime, std::io::Error) {
        (self.serving, self.error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalStoreCloseFailureCause {
    WorkerPanicked,
    OutcomeUnavailable,
}

/// Owning inspect/reopen-only evidence for an accepted task failure.
///
/// No cause certifies that the Store is drained, the lease is held/released, or
/// the Store closed. Existing owner unwinding determines actual resource fate.
#[must_use]
pub struct PhysicalStoreCloseFailure {
    custody: PhysicalCloseTaskCustody,
    cause: PhysicalStoreCloseFailureCause,
    actual: Option<PhysicalStoreCloseOutcome>,
    observation: Option<PhysicalStoreCloseObservation>,
}

impl PhysicalStoreCloseFailure {
    pub(super) fn new(
        custody: PhysicalCloseTaskCustody,
        cause: PhysicalStoreCloseFailureCause,
        actual: Option<PhysicalStoreCloseOutcome>,
        observation: Option<PhysicalStoreCloseObservation>,
    ) -> Self {
        Self {
            custody,
            cause,
            actual,
            observation,
        }
    }

    pub fn runtime_identity(&self) -> RuntimeIdentity {
        self.custody.identity()
    }
    pub fn cause(&self) -> &PhysicalStoreCloseFailureCause {
        &self.cause
    }
    pub fn actual_outcome(&self) -> Option<&PhysicalStoreCloseOutcome> {
        self.actual.as_ref()
    }
    pub fn observation(&self) -> Option<&PhysicalStoreCloseObservation> {
        self.observation.as_ref()
    }

    pub fn into_parts(
        self,
    ) -> (
        PhysicalStoreCloseFailureCause,
        Option<PhysicalStoreCloseOutcome>,
        Option<PhysicalStoreCloseObservation>,
    ) {
        let Self {
            custody,
            cause,
            actual,
            observation,
        } = self;
        custody.failure_evidence_delivered();
        (cause, actual, observation)
    }
}
