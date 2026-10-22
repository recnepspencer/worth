use std::time::{Duration, Instant};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_program::{
        ApplicationArtifactDependency, ApplicationArtifactRetention, ApplicationArtifactSuccession,
        ApplicationComputationExecution, ApplicationComputationInput, ApplicationComputationReuse,
        ApplicationComputationStopped, ApplicationDerivedArtifact, ApplicationFeature,
        ApplicationFeatureInputLeaf, ApplicationLocalityGranule, ApplicationLocalityScope,
        ApplicationManagedComputation, ApplicationOutputPort,
        ApplicationSingleComputationPartition,
    },
    application_schema::{
        ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
        ApplicationStructuredValueBinding,
    },
};

use super::*;
use crate::domain_computation::primary_graph::application_contribution::QueryRequestExecution;

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

#[path = "computation/decision_plan_tests.rs"]
mod decision_plan_tests;
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

    const STOPPED_OUTCOME: &'static str = "worth.query.tests.computation-stopped.v1";
}
struct Input;
impl ApplicationComputationInput for Input {
    type Value = u64;
    const IDENTITY: &'static str = "worth.query.tests.computation-input.v1";
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
    type Partition = ApplicationSingleComputationPartition;
    type Reuse = Reuse;
    type Stopped = Stopped;
    const IDENTITY: &'static str = "worth.query.tests.managed-computation.v1";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::Deterministic;
}
