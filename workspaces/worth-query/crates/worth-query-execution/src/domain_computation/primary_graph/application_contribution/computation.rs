use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_declaration::facade::application_program::{
    ApplicationComputationInput, ApplicationFeature, ApplicationManagedComputation,
};
use worth_query_installation::facade::ApplicationSchema;

pub trait WorthQueryManagedComputationPrepared: Send + 'static {
    fn retained_bytes(&self) -> usize;
}

pub trait WorthQueryManagedComputationOwner<Schema, Feature, Computation>:
    Send + Sync + 'static
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
{
    type Prepared: WorthQueryManagedComputationPrepared;
    type Computed: Send + 'static;
    type Output;
    type Stopped;

    fn prepare(
        &self,
        input: &<Computation::Input as ApplicationComputationInput>::Value,
    ) -> Result<Self::Prepared, Self::Stopped>;
    fn compute(
        &self,
        prepared: &Self::Prepared,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<Self::Computed, WorthQueryManagedComputationDenial<Self::Stopped>>;
    fn complete(
        &self,
        prepared: Self::Prepared,
        computed: Self::Computed,
    ) -> Result<Self::Output, Self::Stopped>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedComputationResourceDenial {
    WorkExhausted,
    RetainedBytesExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedComputationInterruption {
    Cancelled,
    DeadlineExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedComputationCheckpointDenial {
    Resource(WorthQueryManagedComputationResourceDenial),
    Interrupted(WorthQueryManagedComputationInterruption),
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorthQueryManagedComputationDenial<Stopped> {
    Owner(Stopped),
    Resource(WorthQueryManagedComputationResourceDenial),
    Interrupted(WorthQueryManagedComputationInterruption),
}

impl<Stopped> From<WorthQueryManagedComputationCheckpointDenial>
    for WorthQueryManagedComputationDenial<Stopped>
{
    fn from(denial: WorthQueryManagedComputationCheckpointDenial) -> Self {
        match denial {
            WorthQueryManagedComputationCheckpointDenial::Resource(denial) => {
                Self::Resource(denial)
            }
            WorthQueryManagedComputationCheckpointDenial::Interrupted(interruption) => {
                Self::Interrupted(interruption)
            }
        }
    }
}

pub struct WorthQueryManagedComputationExecution<'request> {
    request:
        &'request worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
}

impl<'request> WorthQueryManagedComputationExecution<'request> {
    pub(in crate::domain_computation::primary_graph) const fn new(
        request: &'request worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Self {
        Self { request }
    }

    /// The request's cancellation or elapsed deadline, if either has happened.
    pub(super) fn interruption(&self) -> Option<WorthQueryManagedComputationInterruption> {
        self.request.interruption().map(|interruption| match interruption {
            worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::Cancelled => {
                WorthQueryManagedComputationInterruption::Cancelled
            }
            worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::DeadlineExceeded => {
                WorthQueryManagedComputationInterruption::DeadlineExceeded
            }
        })
    }
}

/// Where a checkpoint's work is charged.
enum CheckpointWork<'request> {
    /// The single partition of a `Deterministic` computation spends the
    /// declared ceiling directly.
    Declared { remaining: usize },
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
    work: CheckpointWork<'request>,
    execution: &'request WorthQueryManagedComputationExecution<'request>,
}

impl<'request> WorthQueryManagedComputationCheckpoint<'request> {
    pub(super) fn for_partition(
        charge: &'request mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        execution: &'request WorthQueryManagedComputationExecution<'request>,
    ) -> Self {
        Self {
            work: CheckpointWork::Partition {
                charge,
                refused: None,
            },
            execution,
        }
    }

    /// The refusal a partition checkpoint kept, if any charge was refused.
    pub(super) const fn refused(&self) -> Option<WorthQueryManagedComputationCheckpointDenial> {
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
            CheckpointWork::Declared { remaining } => {
                if let Some(interruption) = interruption {
                    return Err(interruption);
                }
                *remaining = remaining.checked_sub(work).ok_or(
                    WorthQueryManagedComputationCheckpointDenial::Resource(
                        WorthQueryManagedComputationResourceDenial::WorkExhausted,
                    ),
                )?;
                Ok(())
            }
            CheckpointWork::Partition { charge, refused } => {
                if let Some(denial) = *refused {
                    return Err(denial);
                }
                let outcome = match interruption {
                    Some(interruption) => Err(interruption),
                    None => charge(u64::try_from(work).unwrap_or(u64::MAX)).map_err(|stop| {
                        WorthQueryManagedComputationCheckpointDenial::from_kernel_stop(stop)
                    }),
                };
                *refused = outcome.err();
                outcome
            }
        }
    }
}

impl WorthQueryManagedComputationCheckpointDenial {
    /// What an owner learns from a kernel stop. The kernel keeps the exact
    /// stop and reports it; a stopped nested pattern reads here as exhausted
    /// work because the partition may charge nothing further.
    pub(super) const fn from_kernel_stop(stop: worth_execution::MapKernelStop) -> Self {
        match stop {
            worth_execution::MapKernelStop::Cancelled => {
                Self::Interrupted(WorthQueryManagedComputationInterruption::Cancelled)
            }
            worth_execution::MapKernelStop::DeadlineElapsed => {
                Self::Interrupted(WorthQueryManagedComputationInterruption::DeadlineExceeded)
            }
            worth_execution::MapKernelStop::WorkCeiling
            | worth_execution::MapKernelStop::WorkCounterOverflow
            | worth_execution::MapKernelStop::NestedStopped => {
                Self::Resource(WorthQueryManagedComputationResourceDenial::WorkExhausted)
            }
        }
    }
}

pub struct WorthQueryInstalledManagedComputation<Schema, Feature, Computation, Owner> {
    owner: Arc<Owner>,
    marker: PhantomData<fn() -> (Schema, Feature, Computation)>,
}

impl<Schema, Feature, Computation, Owner> Clone
    for WorthQueryInstalledManagedComputation<Schema, Feature, Computation, Owner>
{
    fn clone(&self) -> Self {
        Self {
            owner: Arc::clone(&self.owner),
            marker: PhantomData,
        }
    }
}

impl<Schema, Feature, Computation, Owner>
    WorthQueryInstalledManagedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryManagedComputationOwner<Schema, Feature, Computation>,
{
    pub(in crate::domain_computation::primary_graph) fn new(owner: Owner) -> Self {
        Self {
            owner: Arc::new(owner),
            marker: PhantomData,
        }
    }

    pub fn prepare(
        &self,
        input: &<Computation::Input as ApplicationComputationInput>::Value,
    ) -> Result<
        WorthQueryPreparedManagedComputation<Schema, Feature, Computation, Owner>,
        WorthQueryManagedComputationDenial<Owner::Stopped>,
    > {
        let prepared = self
            .owner
            .prepare(input)
            .map_err(WorthQueryManagedComputationDenial::Owner)?;
        if prepared.retained_bytes() > Computation::RESOURCES.maximum_retained_bytes() {
            return Err(WorthQueryManagedComputationDenial::Resource(
                WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted,
            ));
        }
        Ok(WorthQueryPreparedManagedComputation {
            installed: self.clone(),
            prepared,
        })
    }
}

#[cfg(test)]
#[path = "computation_tests.rs"]
mod tests;

pub struct WorthQueryPreparedManagedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryManagedComputationOwner<Schema, Feature, Computation>,
{
    installed: WorthQueryInstalledManagedComputation<Schema, Feature, Computation, Owner>,
    prepared: Owner::Prepared,
}

impl<Schema, Feature, Computation, Owner>
    WorthQueryPreparedManagedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryManagedComputationOwner<Schema, Feature, Computation>,
{
    pub fn compute(
        self,
        execution: WorthQueryManagedComputationExecution<'_>,
    ) -> Result<
        WorthQueryCompletedManagedComputation<Schema, Feature, Computation, Owner>,
        WorthQueryManagedComputationDenial<Owner::Stopped>,
    > {
        let mut checkpoint = WorthQueryManagedComputationCheckpoint {
            work: CheckpointWork::Declared {
                remaining: Computation::RESOURCES.maximum_work(),
            },
            execution: &execution,
        };
        let computed = self
            .installed
            .owner
            .compute(&self.prepared, &mut checkpoint)?;
        Ok(WorthQueryCompletedManagedComputation {
            installed: self.installed,
            prepared: self.prepared,
            computed,
        })
    }
}

pub struct WorthQueryCompletedManagedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryManagedComputationOwner<Schema, Feature, Computation>,
{
    installed: WorthQueryInstalledManagedComputation<Schema, Feature, Computation, Owner>,
    prepared: Owner::Prepared,
    computed: Owner::Computed,
}

impl<Schema, Feature, Computation, Owner>
    WorthQueryCompletedManagedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryManagedComputationOwner<Schema, Feature, Computation>,
{
    pub fn complete(
        self,
    ) -> Result<Owner::Output, WorthQueryManagedComputationDenial<Owner::Stopped>> {
        self.installed
            .owner
            .complete(self.prepared, self.computed)
            .map_err(WorthQueryManagedComputationDenial::Owner)
    }
}
