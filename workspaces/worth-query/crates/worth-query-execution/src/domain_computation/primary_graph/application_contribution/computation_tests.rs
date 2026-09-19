use std::time::{Duration, Instant};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
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
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
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
        self.0 as usize
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
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<()>> {
        checkpoint
            .advance(prepared.0 as usize)
            .map_err(WorthQueryManagedComputationDenial::from)?;
        Ok(prepared.0 * 2)
    }
    fn complete(&self, _: Prepared, computed: u64) -> Result<u64, ()> {
        Ok(computed)
    }
}

#[test]
fn installed_owner_enforces_prepare_compute_complete_and_work_ceiling() {
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    let installed =
        WorthQueryInstalledManagedComputation::<Schema, Feature, Computation, Owner>::new(Owner);
    let output = installed
        .prepare(&2)
        .expect("preparation fits")
        .compute(WorthQueryManagedComputationExecution::new(&request))
        .expect("computation fits")
        .complete()
        .expect("completion succeeds");
    assert_eq!(output, 4);
    let denial = match installed
        .prepare(&5)
        .expect("preparation fits")
        .compute(WorthQueryManagedComputationExecution::new(&request))
    {
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

#[test]
fn installed_owner_enforces_retention_and_real_request_interruption() {
    let installed =
        WorthQueryInstalledManagedComputation::<Schema, Feature, Computation, Owner>::new(Owner);
    let denial = match installed.prepare(&65) {
        Err(denial) => denial,
        Ok(_) => panic!("retention above the ceiling is denied"),
    };
    assert_eq!(
        denial,
        WorthQueryManagedComputationDenial::Resource(
            WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted
        )
    );

    let cancellation = WorthQueryCancellationSource::new();
    let cancelled = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    cancellation.cancel();
    let denial = match installed
        .prepare(&1)
        .unwrap()
        .compute(WorthQueryManagedComputationExecution::new(&cancelled))
    {
        Err(denial) => denial,
        Ok(_) => panic!("cancelled execution is denied at its checkpoint"),
    };
    assert_eq!(
        denial,
        WorthQueryManagedComputationDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled
        )
    );

    let deadline_source = WorthQueryCancellationSource::new();
    let expired = WorthQueryRequestScope::new(
        Instant::now() - Duration::from_secs(1),
        deadline_source.token(),
    );
    let denial = match installed
        .prepare(&1)
        .unwrap()
        .compute(WorthQueryManagedComputationExecution::new(&expired))
    {
        Err(denial) => denial,
        Ok(_) => panic!("expired execution is denied at its checkpoint"),
    };
    assert_eq!(
        denial,
        WorthQueryManagedComputationDenial::Interrupted(
            WorthQueryManagedComputationInterruption::DeadlineExceeded
        )
    );
}
