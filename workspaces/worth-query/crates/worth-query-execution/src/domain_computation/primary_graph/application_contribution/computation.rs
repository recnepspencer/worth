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
}

pub struct WorthQueryManagedComputationCheckpoint<'request> {
    remaining_work: usize,
    request:
        &'request worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
}

impl WorthQueryManagedComputationCheckpoint<'_> {
    pub fn advance(
        &mut self,
        work: usize,
    ) -> Result<(), WorthQueryManagedComputationCheckpointDenial> {
        if let Some(interruption) = self.request.interruption() {
            let interruption = match interruption {
                worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::Cancelled => {
                    WorthQueryManagedComputationInterruption::Cancelled
                }
                worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::DeadlineExceeded => {
                    WorthQueryManagedComputationInterruption::DeadlineExceeded
                }
            };
            return Err(WorthQueryManagedComputationCheckpointDenial::Interrupted(
                interruption,
            ));
        }
        self.remaining_work = self.remaining_work.checked_sub(work).ok_or(
            WorthQueryManagedComputationCheckpointDenial::Resource(
                WorthQueryManagedComputationResourceDenial::WorkExhausted,
            ),
        )?;
        Ok(())
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
            remaining_work: Computation::RESOURCES.maximum_work(),
            request: execution.request,
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
