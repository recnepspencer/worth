//! Where a managed computation's checkpoints charge, and what they observe.

use super::super::execution_denial::checkpoint_denial;
use super::super::request_execution::QueryRequestExecution;
use super::{
    WorthQueryManagedComputationCheckpointDenial, WorthQueryManagedComputationInterruption,
};

/// The request a managed computation runs for: its cancellation, its
/// deadline and how its patterns are placed.
pub struct WorthQueryManagedComputationExecution<'request> {
    pub(crate) request: &'request QueryRequestExecution<'request>,
}

impl<'request> WorthQueryManagedComputationExecution<'request> {
    pub const fn new(request: &'request QueryRequestExecution<'request>) -> Self {
        Self { request }
    }

    /// The request's cancellation or elapsed deadline, if either has happened.
    pub fn interruption(&self) -> Option<WorthQueryManagedComputationInterruption> {
        self.request.interruption()
    }
}

pub struct WorthQueryManagedComputationCheckpoint<'request> {
    pub(crate) charge:
        Option<&'request mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>>,
    pub(crate) poll: &'request mut dyn FnMut() -> Result<(), worth_execution::MapKernelStop>,
    pub(crate) refused: Option<WorthQueryManagedComputationCheckpointDenial>,
    pub(crate) native: Option<&'request worth_execution::ExecutionResourceLease<'request>>,
}

impl<'request> WorthQueryManagedComputationCheckpoint<'request> {
    pub fn with_native_lease_for_test(
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
    pub(crate) fn for_partition(
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
    pub const fn refused(&self) -> Option<WorthQueryManagedComputationCheckpointDenial> {
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
                super::WorthQueryManagedComputationResourceDenial::AccountingUnavailable,
            )),
        };
        self.refused = result.err();
        result
    }
}

mod court_doors_for_test;

/// Crate-boundary internal port for `WorthQueryManagedComputationExecution`.
/// Never exported on `worth-query-execution::facade` or Host.
pub trait ManagedComputationExecutionInternalPort<'request>: Sized {
    fn from_request(request: &'request QueryRequestExecution<'request>) -> Self;
    fn request_ref(&self) -> &'request QueryRequestExecution<'request>;
}

impl<'request> ManagedComputationExecutionInternalPort<'request>
    for WorthQueryManagedComputationExecution<'request>
{
    fn from_request(request: &'request QueryRequestExecution<'request>) -> Self {
        Self::new(request)
    }

    fn request_ref(&self) -> &'request QueryRequestExecution<'request> {
        self.request
    }
}

/// Crate-boundary internal port for `WorthQueryManagedComputationCheckpoint`.
/// Never exported on `worth-query-execution::facade` or Host.
pub trait ManagedComputationCheckpointInternalPort<'request>: Sized {
    fn for_partition(
        charge: Option<&'request mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>>,
        poll: &'request mut dyn FnMut() -> Result<(), worth_execution::MapKernelStop>,
    ) -> Self;
    fn with_native_lease(
        self,
        lease: Option<&'request worth_execution::ExecutionResourceLease<'request>>,
    ) -> Self;
    fn native_lease(&self) -> Option<&'request worth_execution::ExecutionResourceLease<'request>>;
}

impl<'request> ManagedComputationCheckpointInternalPort<'request>
    for WorthQueryManagedComputationCheckpoint<'request>
{
    fn for_partition(
        charge: Option<&'request mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>>,
        poll: &'request mut dyn FnMut() -> Result<(), worth_execution::MapKernelStop>,
    ) -> Self {
        Self::for_partition(charge, poll)
    }

    fn with_native_lease(
        mut self,
        lease: Option<&'request worth_execution::ExecutionResourceLease<'request>>,
    ) -> Self {
        self.native = lease;
        self
    }

    fn native_lease(&self) -> Option<&'request worth_execution::ExecutionResourceLease<'request>> {
        self.native
    }
}
