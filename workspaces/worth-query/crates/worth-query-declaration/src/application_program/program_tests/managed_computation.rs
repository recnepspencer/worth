use super::*;
use crate::application_program::{
    ApplicationArtifactDependency, ApplicationArtifactResourceCeiling,
    ApplicationArtifactRetention, ApplicationArtifactSuccession, ApplicationCollectionContributor,
    ApplicationCollectionGrouping, ApplicationCollectionIncompletePosture,
    ApplicationCollectionLineage, ApplicationCollectionMeasures, ApplicationComputationExecution,
    ApplicationComputationInput, ApplicationComputationPartition,
    ApplicationComputationResourceCeiling, ApplicationComputationReuse,
    ApplicationComputationStopped, ApplicationDerivedArtifact, ApplicationDerivedCollection,
    ApplicationLocalityGranule, ApplicationLocalityScope, ApplicationManagedComputation,
    ApplicationProgramValidationDenialKind,
};

struct Locality;
impl ApplicationLocalityScope for Locality {
    const IDENTITY: &'static str = "worth.query.tests.computation-locality.v1";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Partition;
}

struct Artifact;
impl ApplicationDerivedArtifact<TestSchema, FlatFeature> for Artifact {
    type Output = Export;
    type Locality = Locality;
    const IDENTITY: &'static str = "worth.query.tests.computation-artifact.v1";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Disposable;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str = "worth.query.tests.computation-producer.v1";
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] =
        &[ApplicationArtifactDependency::new(FlatFeature::IDENTITY)];
    const REUSE_RULE: &'static str = "worth.query.tests.computation-reuse.v1";
    const RESOURCE_CEILING: ApplicationArtifactResourceCeiling =
        ApplicationArtifactResourceCeiling::new(8, 4_096);
    const STOPPED_OUTCOME: &'static str = "worth.query.tests.computation-artifact-stopped.v1";
}

struct InputMeaning;
impl ApplicationComputationInput for InputMeaning {
    type Value = u64;
    const IDENTITY: &'static str = "worth.query.tests.computation-input.v1";
}
struct Partition;
impl ApplicationComputationPartition for Partition {
    const IDENTITY: &'static str = "worth.query.tests.computation-partition.v1";
}
struct Reuse;
impl ApplicationComputationReuse for Reuse {
    const IDENTITY: &'static str = "worth.query.tests.computation-reuse-evidence.v1";
}
struct Stopped;
impl ApplicationComputationStopped for Stopped {
    const IDENTITY: &'static str = "worth.query.tests.computation-stopped.v1";
}
struct Computation;
impl ApplicationManagedComputation<TestSchema, FlatFeature> for Computation {
    type Input = InputMeaning;
    type Output = Artifact;
    type Partition = Partition;
    type Reuse = Reuse;
    type Stopped = Stopped;
    const IDENTITY: &'static str = "worth.query.tests.managed-computation.v1";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
    const ORDERING: &'static str = "worth.query.tests.partition-order.v1";
    const RESOURCES: ApplicationComputationResourceCeiling =
        ApplicationComputationResourceCeiling::new(16, 8_192);
}

struct Contributor;
impl ApplicationCollectionContributor for Contributor {
    const IDENTITY: &'static str = "worth.query.tests.collection-contributor.v1";
}
struct Grouping;
impl ApplicationCollectionGrouping for Grouping {
    const IDENTITY: &'static str = "worth.query.tests.collection-grouping.v1";
}
struct Measures;
impl ApplicationCollectionMeasures for Measures {
    const IDENTITY: &'static str = "worth.query.tests.collection-measures.v1";
}
struct Lineage;
impl ApplicationCollectionLineage for Lineage {
    const IDENTITY: &'static str = "worth.query.tests.collection-lineage.v1";
}
struct Collection;
impl ApplicationDerivedCollection<TestSchema, FlatFeature> for Collection {
    type Contributor = Contributor;
    type Grouping = Grouping;
    type Measures = Measures;
    type Lineage = Lineage;
    const IDENTITY: &'static str = "worth.query.tests.derived-collection.v1";
    const APPLICABILITY: &'static str = "worth.query.tests.collection-applicability.v1";
    const INCOMPLETE: ApplicationCollectionIncompletePosture =
        ApplicationCollectionIncompletePosture::RetainPending;
    const INCREMENTAL_UPDATE: &'static str = "worth.query.tests.collection-exact-delta.v1";
}

struct Program;
impl ApplicationProgramDefinition<TestSchema> for Program {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.tests.computation-program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<TestSchema, FlatFeature>()
            .derived_artifact::<Artifact>()
            .managed_computation::<Computation>()
            .derived_collection::<Collection>()
            .finish()]
    }
}

#[test]
fn derived_collection_retains_typed_grouping_measure_and_lineage_meaning() {
    let program = ApplicationProgramAuthoring::<TestSchema, Program>::begin()
        .validated_program()
        .expect("derived collection validates");
    let collection = &program.features()[0].derived_collections()[0];
    assert_eq!(collection.identity(), Collection::IDENTITY);
    assert_eq!(collection.contributor(), Contributor::IDENTITY);
    assert_eq!(collection.grouping(), Grouping::IDENTITY);
    assert_eq!(collection.measures(), Measures::IDENTITY);
    assert_eq!(collection.lineage(), Lineage::IDENTITY);
    assert_eq!(
        collection.incomplete(),
        ApplicationCollectionIncompletePosture::RetainPending
    );
}

#[test]
fn computation_references_its_attached_artifact_without_copying_lifecycle_policy() {
    let program = ApplicationProgramAuthoring::<TestSchema, Program>::begin()
        .validated_program()
        .expect("managed computation validates");
    let computation = &program.features()[0].managed_computations()[0];
    assert_eq!(computation.identity(), Computation::IDENTITY);
    assert_eq!(computation.output_artifact(), Artifact::IDENTITY);
    assert_eq!(computation.resources(), Computation::RESOURCES);
    assert_eq!(
        computation.execution(),
        ApplicationComputationExecution::DeterministicPartitioned
    );
}

#[test]
fn computation_without_its_output_artifact_is_denied() {
    struct MissingArtifactProgram;
    impl ApplicationProgramDefinition<TestSchema> for MissingArtifactProgram {
        type Contributions = ();
        type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
        type Rules = ApplicationRuleLeaf;
        const IDENTITY: ApplicationProgramIdentity =
            ApplicationProgramIdentity::new("worth.query.tests.missing-computation-artifact.v1");
        fn feature_specs() -> Vec<ApplicationFeatureSpec> {
            vec![ApplicationFeatureSpec::root::<TestSchema, FlatFeature>()
                .managed_computation::<Computation>()
                .finish()]
        }
    }
    let denial = ApplicationProgramAuthoring::<TestSchema, MissingArtifactProgram>::begin()
        .validated_program()
        .err()
        .expect("missing output artifact must be denied");
    assert_eq!(
        denial.kind(),
        ApplicationProgramValidationDenialKind::MissingManagedComputationArtifact
    );
}
