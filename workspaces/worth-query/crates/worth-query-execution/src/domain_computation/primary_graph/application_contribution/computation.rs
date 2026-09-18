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
        checkpoint: &mut WorthQueryManagedComputationCheckpoint,
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

#[derive(Debug, Eq, PartialEq)]
pub enum WorthQueryManagedComputationDenial<Stopped> {
    Owner(Stopped),
    Resource(WorthQueryManagedComputationResourceDenial),
}

pub struct WorthQueryManagedComputationCheckpoint {
    remaining_work: usize,
}

impl WorthQueryManagedComputationCheckpoint {
    pub fn advance(
        &mut self,
        work: usize,
    ) -> Result<(), WorthQueryManagedComputationResourceDenial> {
        self.remaining_work = self
            .remaining_work
            .checked_sub(work)
            .ok_or(WorthQueryManagedComputationResourceDenial::WorkExhausted)?;
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
mod tests {
    use worth_query_declaration::facade::{
        application_program::{
            ApplicationArtifactDependency, ApplicationArtifactResourceCeiling,
            ApplicationArtifactRetention, ApplicationArtifactSuccession,
            ApplicationComputationExecution, ApplicationComputationInput,
            ApplicationComputationPartition, ApplicationComputationResourceCeiling,
            ApplicationComputationReuse, ApplicationComputationStopped, ApplicationDerivedArtifact,
            ApplicationFeature, ApplicationFeatureInputLeaf, ApplicationLocalityGranule,
            ApplicationLocalityScope, ApplicationManagedComputation, ApplicationOutputPort,
        },
        application_schema::{
            ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
            ApplicationStructuredValueBinding,
        },
    };

    use super::*;

    struct Schema;
    impl ApplicationSchema for Schema {
        const OWNER: &'static str = "worth.query.tests";
        const NAME: &'static str = "managed-computation";
        const MAJOR: u32 = 1;
        const MINOR: u32 = 0;
        fn declaration(
        ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial>
        {
            unreachable!("the execution phase proof needs no schema installation")
        }
    }
    struct Feature;
    impl ApplicationFeature<Schema> for Feature {
        type Inputs = ApplicationFeatureInputLeaf;
        const IDENTITY: &'static str = "worth.query.tests.computation-feature.v1";
    }
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct Value(u64);
    worth_query_declaration::worth_query_portable_type!(
        Value => "worth.query.tests.computation-value.v1"
    );
    struct ValueBinding;
    impl ApplicationStructuredValueBinding for ValueBinding {
        type Value = Value;
        const IDENTITY_NAME: &'static str = "worth.query.tests.computation-value.v1";
        fn validate(
            _: &Self::Value,
        ) -> Result<
            (),
            worth_query_declaration::facade::application_schema::ApplicationValueValidationDenial,
        > {
            Ok(())
        }
    }
    struct Output;
    impl ApplicationOutputPort<Schema, Feature> for Output {
        type Value = ValueBinding;
        const IDENTITY: &'static str = "worth.query.tests.computation-output.v1";
    }
    struct Locality;
    impl ApplicationLocalityScope for Locality {
        const IDENTITY: &'static str = "worth.query.tests.computation-locality.v1";
        const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Partition;
    }
    struct Artifact;
    impl ApplicationDerivedArtifact<Schema, Feature> for Artifact {
        type Output = Output;
        type Locality = Locality;
        const IDENTITY: &'static str = "worth.query.tests.computation-artifact.v1";
        const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Disposable;
        const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
        const REQUIRED: bool = true;
        const PRODUCER_FAMILY: &'static str = "worth.query.tests.computation-producer.v1";
        const DEPENDENCIES: &'static [ApplicationArtifactDependency] = &[];
        const REUSE_RULE: &'static str = "worth.query.tests.computation-reuse.v1";
        const RESOURCE_CEILING: ApplicationArtifactResourceCeiling =
            ApplicationArtifactResourceCeiling::new(4, 64);
        const STOPPED_OUTCOME: &'static str = "worth.query.tests.computation-stopped.v1";
    }
    struct Input;
    impl ApplicationComputationInput for Input {
        type Value = u64;
        const IDENTITY: &'static str = "worth.query.tests.computation-input.v1";
    }
    struct Partition;
    impl ApplicationComputationPartition for Partition {
        const IDENTITY: &'static str = "worth.query.tests.computation-partition.v1";
    }
    struct Reuse;
    impl ApplicationComputationReuse for Reuse {
        const IDENTITY: &'static str = "worth.query.tests.computation-evidence.v1";
    }
    struct Stopped;
    impl ApplicationComputationStopped for Stopped {
        const IDENTITY: &'static str = "worth.query.tests.computation-owner-stopped.v1";
    }
    struct Computation;
    impl ApplicationManagedComputation<Schema, Feature> for Computation {
        type Input = Input;
        type Output = Artifact;
        type Partition = Partition;
        type Reuse = Reuse;
        type Stopped = Stopped;
        const IDENTITY: &'static str = "worth.query.tests.managed-computation.v1";
        const EXECUTION: ApplicationComputationExecution =
            ApplicationComputationExecution::Deterministic;
        const ORDERING: &'static str = "worth.query.tests.computation-order.v1";
        const RESOURCES: ApplicationComputationResourceCeiling =
            ApplicationComputationResourceCeiling::new(4, 64);
    }
    struct Prepared(u64);
    impl WorthQueryManagedComputationPrepared for Prepared {
        fn retained_bytes(&self) -> usize {
            std::mem::size_of::<Self>()
        }
    }
    struct Owner;
    impl WorthQueryManagedComputationOwner<Schema, Feature, Computation> for Owner {
        type Prepared = Prepared;
        type Computed = u64;
        type Output = u64;
        type Stopped = ();
        fn prepare(&self, input: &u64) -> Result<Prepared, ()> {
            Ok(Prepared(*input))
        }
        fn compute(
            &self,
            prepared: &Prepared,
            checkpoint: &mut WorthQueryManagedComputationCheckpoint,
        ) -> Result<u64, WorthQueryManagedComputationDenial<()>> {
            checkpoint
                .advance(prepared.0 as usize)
                .map_err(WorthQueryManagedComputationDenial::Resource)?;
            Ok(prepared.0 * 2)
        }
        fn complete(&self, _: Prepared, computed: u64) -> Result<u64, ()> {
            Ok(computed)
        }
    }

    #[test]
    fn installed_owner_enforces_prepare_compute_complete_and_work_ceiling() {
        let installed =
            WorthQueryInstalledManagedComputation::<Schema, Feature, Computation, Owner>::new(
                Owner,
            );
        let output = installed
            .prepare(&2)
            .expect("preparation fits")
            .compute()
            .expect("computation fits")
            .complete()
            .expect("completion succeeds");
        assert_eq!(output, 4);
        let denial = match installed.prepare(&5).expect("preparation fits").compute() {
            Err(denial) => denial,
            Ok(_) => panic!("work above the declared ceiling must be denied"),
        };
        assert_eq!(
            denial,
            WorthQueryManagedComputationDenial::Resource(
                WorthQueryManagedComputationResourceDenial::WorkExhausted
            )
        );
    }
}

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
    ) -> Result<
        WorthQueryCompletedManagedComputation<Schema, Feature, Computation, Owner>,
        WorthQueryManagedComputationDenial<Owner::Stopped>,
    > {
        let mut checkpoint = WorthQueryManagedComputationCheckpoint {
            remaining_work: Computation::RESOURCES.maximum_work(),
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
