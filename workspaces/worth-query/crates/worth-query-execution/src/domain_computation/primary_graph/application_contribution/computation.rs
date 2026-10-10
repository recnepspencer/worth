use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_declaration::facade::application_program::{
    ApplicationComputationInput, ApplicationFeature, ApplicationManagedComputation,
};
use worth_query_installation::facade::ApplicationSchema;

/// Prepared values and the transported owner are the handler author's own data.
/// An author-held runtime handle used from compute follows the worker reentry rule.
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
    type Stopped: Send + 'static;

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

mod checkpoint;
mod member_execution;
pub(in crate::domain_computation::primary_graph) mod nested_failure;
pub(in crate::domain_computation::primary_graph) mod worker_custody;
pub use member_execution::WorthQueryMemberExecution;
mod decision_plan;
mod denial;
mod preparation_loans;
pub(in crate::domain_computation::primary_graph) mod prepared_request;
pub(in crate::domain_computation::primary_graph) mod wave_dispatch;

use checkpoint::CheckpointWork;
pub use checkpoint::{
    WorthQueryManagedComputationCheckpoint, WorthQueryManagedComputationExecution,
};
pub use denial::{
    WorthQueryManagedComputationCheckpointDenial, WorthQueryManagedComputationDenial,
    WorthQueryManagedComputationInterruption, WorthQueryManagedComputationResourceDenial,
    WorthQueryMemoryLimitLevel,
};

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
        self.compute_with_charge(execution, None)
    }

    fn compute_for_member(
        self,
        member: WorthQueryMemberExecution<'_>,
    ) -> Result<
        WorthQueryCompletedManagedComputation<Schema, Feature, Computation, Owner>,
        WorthQueryManagedComputationDenial<Owner::Stopped>,
    > {
        member.compute(|execution, charge, _transport| self.compute_with_charge(execution, charge))
    }

    fn compute_with_charge<'a>(
        self,
        execution: WorthQueryManagedComputationExecution<'a>,
        mut charge: Option<&'a mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>>,
    ) -> Result<
        WorthQueryCompletedManagedComputation<Schema, Feature, Computation, Owner>,
        WorthQueryManagedComputationDenial<Owner::Stopped>,
    > {
        let mut delegated = |units| match &mut charge {
            Some(charge) => charge(units),
            None => Ok(()),
        };
        let mut checkpoint = WorthQueryManagedComputationCheckpoint {
            work: CheckpointWork::Declared {
                remaining: Computation::RESOURCES.maximum_work(),
                charge: Some(&mut delegated),
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
