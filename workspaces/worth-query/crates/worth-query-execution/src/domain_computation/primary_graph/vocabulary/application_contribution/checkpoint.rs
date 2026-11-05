//! Moved whole from `domain_computation::primary_graph::application_contribution::computation::checkpoint`: the items below and every impl they own.
use crate::domain_computation::request_execution::QueryRequestExecution;
use crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationCheckpointDenial;
use crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationInterruption;
use crate::domain_computation::request_execution::execution_stop::checkpoint_denial;

/// The request a managed computation runs for: its cancellation, its
/// deadline and how its patterns are placed.
pub struct WorthQueryManagedComputationExecution<'request> {
    pub(in crate::domain_computation::primary_graph) request: &'request QueryRequestExecution<'request>,
}

impl<'request> WorthQueryManagedComputationExecution<'request> {
    pub const fn new(request: &'request QueryRequestExecution<'request>) -> Self {
        Self { request }
    }

    /// The request's cancellation or elapsed deadline, if either has happened.
    pub(in crate::domain_computation::primary_graph) fn interruption(
        &self,
    ) -> Option<WorthQueryManagedComputationInterruption> {
        self.request.interruption()
    }
}

pub struct WorthQueryManagedComputationCheckpoint<'request> {
    pub(in crate::domain_computation::primary_graph) charge:
        Option<&'request mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>>,
    pub(in crate::domain_computation::primary_graph) poll: &'request mut dyn FnMut() -> Result<(), worth_execution::MapKernelStop>,
    pub(in crate::domain_computation::primary_graph) refused: Option<WorthQueryManagedComputationCheckpointDenial>,
    pub(in crate::domain_computation::primary_graph) native: Option<&'request worth_execution::ExecutionResourceLease<'request>>,
}

impl<'request> WorthQueryManagedComputationCheckpoint<'request> {
    pub(in crate::domain_computation::primary_graph) fn with_native_lease_for_test(
        mut self,
        lease: Option<&'request worth_execution::ExecutionResourceLease<'request>>,
    ) -> Self {
        self.native = lease;
        self
    }
    pub fn native_lease_for_test(
        &self,
    ) -> Option<&'request worth_execution::ExecutionResourceLease<'request>> {
        crate::domain_computation::execution_observation::owned_control_scope::current_id();
        self.native
    }
    pub(in crate::domain_computation::primary_graph) fn for_partition(
        charge: Option<&'request mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>>,
        poll: &'request mut dyn FnMut() -> Result<(), worth_execution::MapKernelStop>,
    ) -> Self {
        Self {
            charge,
            poll,
            refused: None,
            native: None,
        }
    }

    /// The refusal a partition checkpoint kept, if any charge was refused.
    pub(in crate::domain_computation::primary_graph) const fn refused(
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
        let result = match self.charge.as_mut() {
            Some(charge) => {
                charge(u64::try_from(work).unwrap_or(u64::MAX)).map_err(checkpoint_denial)
            }
            None => Err(WorthQueryManagedComputationCheckpointDenial::Resource(
                crate::domain_computation::vocabulary::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial::AccountingUnavailable,
            )),
        };
        self.refused = result.err();
        result
    }
}

impl<'request> WorthQueryManagedComputationCheckpoint<'request> {
    #[doc(hidden)]
    pub fn charge_for_test(
        &self,
    ) -> &Option<&'request mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>> {
        &self.charge
    }
}

impl<'request> WorthQueryManagedComputationExecution<'request> {
    #[doc(hidden)]
    pub fn request_for_test(&self) -> &&'request QueryRequestExecution<'request> {
        &self.request
    }
}
