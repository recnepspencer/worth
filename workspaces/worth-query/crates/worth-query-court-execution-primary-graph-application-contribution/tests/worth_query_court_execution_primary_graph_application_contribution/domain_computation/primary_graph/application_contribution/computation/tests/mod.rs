pub use std::time::Duration;
pub use std::time::Instant;
pub use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
pub use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
pub use worth_query_declaration::facade::application_program::ApplicationArtifactDependency;
pub use worth_query_declaration::facade::application_program::ApplicationArtifactRetention;
pub use worth_query_declaration::facade::application_program::ApplicationArtifactSuccession;
pub use worth_query_declaration::facade::application_program::ApplicationComputationExecution;
pub use worth_query_declaration::facade::application_program::ApplicationComputationInput;
pub use worth_query_declaration::facade::application_program::ApplicationComputationReuse;
pub use worth_query_declaration::facade::application_program::ApplicationComputationStopped;
pub use worth_query_declaration::facade::application_program::ApplicationDerivedArtifact;
pub use worth_query_declaration::facade::application_program::ApplicationFeature;
pub use worth_query_declaration::facade::application_program::ApplicationFeatureInputLeaf;
pub use worth_query_declaration::facade::application_program::ApplicationLocalityGranule;
pub use worth_query_declaration::facade::application_program::ApplicationLocalityScope;
pub use worth_query_declaration::facade::application_program::ApplicationManagedComputation;
pub use worth_query_declaration::facade::application_program::ApplicationOutputPort;
pub use worth_query_declaration::facade::application_program::ApplicationSingleComputationPartition;
pub use worth_query_declaration::facade::application_schema::ApplicationSchema;
pub use worth_query_declaration::facade::application_schema::ApplicationSchemaDeclaration;
pub use worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial;
pub use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;

pub use std::sync::Arc;
pub use worth_query_execution::facade::application_contribution::WorthQueryInstalledManagedComputation;
pub use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationCheckpoint;
pub use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationCheckpointDenial;
pub use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationDenial;
pub use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationInterruption;
pub use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial;
pub use worth_query_execution::court_doors_for_test::domain_computation::request_execution::QueryRequestExecution;

pub struct Schema;
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

pub struct Feature;
impl ApplicationFeature<Schema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.tests.computation-feature.v1";
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Value(u64);
worth_query_declaration::worth_query_portable_type!(
    Value => "worth.query.tests.computation-value.v1"
);
pub struct ValueBinding;
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
pub struct Output;
impl ApplicationOutputPort<Schema, Feature> for Output {
    type Value = ValueBinding;
    const IDENTITY: &'static str = "worth.query.tests.computation-output.v1";
}
pub struct Locality;
impl ApplicationLocalityScope for Locality {
    const IDENTITY: &'static str = "worth.query.tests.computation-locality.v1";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Partition;
}
pub struct Artifact;
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

    const STOPPED_OUTCOME: &'static str = "worth.query.tests.computation-stopped.v1";
}
pub struct Input;
impl ApplicationComputationInput for Input {
    type Value = u64;
    const IDENTITY: &'static str = "worth.query.tests.computation-input.v1";
}
pub struct Reuse;
impl ApplicationComputationReuse for Reuse {
    const IDENTITY: &'static str = "worth.query.tests.computation-evidence.v1";
}
pub struct Stopped;
impl ApplicationComputationStopped for Stopped {
    const IDENTITY: &'static str = "worth.query.tests.computation-owner-stopped.v1";
}
pub struct Computation;
impl ApplicationManagedComputation<Schema, Feature> for Computation {
    type Input = Input;
    type Output = Artifact;
    type Partition = ApplicationSingleComputationPartition;
    type Reuse = Reuse;
    type Stopped = Stopped;
    const IDENTITY: &'static str = "worth.query.tests.managed-computation.v1";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::Deterministic;
}

#[path = "decision_plan_tests/mod.rs"]
pub mod decision_plan_tests;
#[path = "member_accounting_tests/mod.rs"]
pub mod member_accounting_tests;
