//! Where a managed computation's checkpoints charge, and what they observe.

use super::super::execution_denial::checkpoint_denial;
use super::super::request_execution::QueryRequestExecution;
use super::{
    WorthQueryManagedComputationCheckpointDenial, WorthQueryManagedComputationInterruption,
    WorthQueryManagedComputationResourceDenial,
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

/// Where a checkpoint's work is charged.
pub(super) enum CheckpointWork<'request> {
    /// The single partition of a `Deterministic` computation spends the
    /// declared ceiling directly.
    Declared {
        remaining: usize,
        charge: Option<&'request mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>>,
    },
    /// One partition of a `DeterministicPartitioned` computation charges the
    /// execution kernel it runs in, which owns the remaining ceiling. A
    /// refused charge is final: it is kept, and the partition reports it
    /// whatever the owner returns afterwards.
    Partition {
        charge: &'request mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        refused: Option<WorthQueryManagedComputationCheckpointDenial>,
    },
}

pub struct WorthQueryManagedComputationCheckpoint<'request> {
    pub(super) work: CheckpointWork<'request>,
    pub(super) execution: &'request WorthQueryManagedComputationExecution<'request>,
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
        execution: &'request WorthQueryManagedComputationExecution<'request>,
    ) -> Self {
        Self {
            work: CheckpointWork::Partition {
                charge,
                refused: None,
            },
            execution,
            #[cfg(test)]
            native: None,
        }
    }

    /// The refusal a partition checkpoint kept, if any charge was refused.
    pub(in super::super) const fn refused(
        &self,
    ) -> Option<WorthQueryManagedComputationCheckpointDenial> {
        match &self.work {
            CheckpointWork::Declared { .. } => None,
            CheckpointWork::Partition { refused, .. } => *refused,
        }
    }

    pub fn advance(
        &mut self,
        work: usize,
    ) -> Result<(), WorthQueryManagedComputationCheckpointDenial> {
        let interruption = self
            .execution
            .interruption()
            .map(WorthQueryManagedComputationCheckpointDenial::Interrupted);
        match &mut self.work {
            CheckpointWork::Declared { remaining, charge } => {
                if let Some(interruption) = interruption {
                    return Err(interruption);
                }
                *remaining = remaining.checked_sub(work).ok_or(
                    WorthQueryManagedComputationCheckpointDenial::Resource(
                        WorthQueryManagedComputationResourceDenial::WorkExhausted,
                    ),
                )?;
                if let Some(charge) = charge {
                    charge(u64::try_from(work).unwrap_or(u64::MAX)).map_err(checkpoint_denial)?;
                }
                Ok(())
            }
            CheckpointWork::Partition { charge, refused } => {
                if let Some(denial) = *refused {
                    return Err(denial);
                }
                let outcome = match interruption {
                    Some(interruption) => Err(interruption),
                    None => {
                        charge(u64::try_from(work).unwrap_or(u64::MAX)).map_err(checkpoint_denial)
                    }
                };
                *refused = outcome.err();
                outcome
            }
        }
    }
}
