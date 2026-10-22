//! Where a managed computation's checkpoints charge, and what they observe.

use super::super::execution_denial::checkpoint_denial;
use super::super::request_execution::QueryRequestExecution;
use super::{
    WorthQueryManagedComputationCheckpointDenial, WorthQueryManagedComputationInterruption,
};

/// The request a managed computation runs for: its cancellation, its
/// deadline and how its patterns are placed.
pub struct WorthQueryManagedComputationExecution<'request> {
    pub(in super::super) request: &'request QueryRequestExecution<'request>,
}

impl<'request> WorthQueryManagedComputationExecution<'request> {
    pub(in crate::domain_computation::primary_graph) const fn new(
        request: &'request QueryRequestExecution<'request>,
    ) -> Self {
        Self { request }
    }

    /// The request's cancellation or elapsed deadline, if either has happened.
    pub(in super::super) fn interruption(
        &self,
    ) -> Option<WorthQueryManagedComputationInterruption> {
        self.request.interruption()
    }
}

pub struct WorthQueryManagedComputationCheckpoint<'request> {
    pub(super) charge: &'request mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    pub(super) poll: &'request mut dyn FnMut() -> Result<(), worth_execution::MapKernelStop>,
    pub(super) refused: Option<WorthQueryManagedComputationCheckpointDenial>,
    #[cfg(test)]
    pub(super) native: Option<&'request worth_execution::ExecutionResourceLease<'request>>,
}

impl<'request> WorthQueryManagedComputationCheckpoint<'request> {
    #[cfg(test)]
    pub(in super::super) fn with_native_lease_for_test(
        mut self,
        lease: Option<&'request worth_execution::ExecutionResourceLease<'request>>,
    ) -> Self {
        self.native = lease;
        self
    }
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn native_lease_for_test(
        &self,
    ) -> Option<&'request worth_execution::ExecutionResourceLease<'request>> {
        crate::domain_computation::execution_observation::owned_control_scope::current_id();
        self.native
    }
    pub(in super::super) fn for_partition(
        charge: &'request mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        poll: &'request mut dyn FnMut() -> Result<(), worth_execution::MapKernelStop>,
    ) -> Self {
        Self {
            charge,
            poll,
            refused: None,
            #[cfg(test)]
            native: None,
        }
    }

    /// The refusal a partition checkpoint kept, if any charge was refused.
    pub(in super::super) const fn refused(
        &self,
    ) -> Option<WorthQueryManagedComputationCheckpointDenial> {
        self.refused
    }

    /// Observe this computation's original request and nested stop without
    /// charging work. A refused safe point stays refused through completion.
    pub fn poll(&mut self) -> Result<(), WorthQueryManagedComputationCheckpointDenial> {
        if let Some(cause) = self.refused {
            return Err(cause);
        }
        let result = (self.poll)().map_err(checkpoint_denial);
        self.refused = result.err();
        result
    }

    pub fn advance(
        &mut self,
        work: usize,
    ) -> Result<(), WorthQueryManagedComputationCheckpointDenial> {
        self.poll()?;
        let result =
            (self.charge)(u64::try_from(work).unwrap_or(u64::MAX)).map_err(checkpoint_denial);
        self.refused = result.err();
        result
    }
}
