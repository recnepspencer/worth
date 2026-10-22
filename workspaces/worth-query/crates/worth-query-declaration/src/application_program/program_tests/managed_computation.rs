use super::*;
use crate::application_program::{
    ApplicationArtifactDependency, ApplicationArtifactRetention, ApplicationArtifactSuccession,
    ApplicationCollectionContributor, ApplicationCollectionGrouping,
    ApplicationCollectionIncompletePosture, ApplicationCollectionLineage,
    ApplicationCollectionMeasures, ApplicationComputationExecution, ApplicationComputationInput,
    ApplicationComputationPartition, ApplicationComputationReuse, ApplicationComputationStopped,
    ApplicationDerivedArtifact, ApplicationDerivedCollection, ApplicationLocalityGranule,
    ApplicationLocalityScope, ApplicationManagedComputation,
    ApplicationProgramValidationDenialKind, ApplicationSingleComputationPartition,
    DeterminismContract,
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

    const STOPPED_OUTCOME: &'static str = "worth.query.tests.computation-artifact-stopped.v1";
}

struct InputMeaning;
impl ApplicationComputationInput for InputMeaning {
    type Value = u64;
    const IDENTITY: &'static str = "worth.query.tests.computation-input.v1";
}
#[derive(Eq, Ord, PartialEq, PartialOrd, serde::Serialize)]
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
macro_rules! computation {
    ($computation:ident, $partition:ty, $execution:expr) => {
        struct $computation;
        impl ApplicationManagedComputation<TestSchema, FlatFeature> for $computation {
            type Input = InputMeaning;
            type Output = Artifact;
            type Partition = $partition;
            type Reuse = Reuse;
            type Stopped = Stopped;
            const IDENTITY: &'static str = "worth.query.tests.managed-computation.v1";
            const EXECUTION: ApplicationComputationExecution = $execution;
        }
    };
}
const PARTITIONED: ApplicationComputationExecution =
    ApplicationComputationExecution::DeterministicPartitioned;
computation!(Computation, Partition, PARTITIONED);
computation!(
    SerialComputation,
    ApplicationSingleComputationPartition,
    ApplicationComputationExecution::Deterministic
);
computation!(
    KeyedSerialComputation,
    Partition,
    ApplicationComputationExecution::Deterministic
);
computation!(
    SinglePartitionedComputation,
    ApplicationSingleComputationPartition,
    PARTITIONED
);

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
struct Completeness;
impl crate::application_program::ApplicationCollectionCompleteness for Completeness {
    type Value = u64;
    type Stopped = u32;
    type Lineage = Lineage;
    const IDENTITY: &'static str = "worth.query.tests.collection-completeness.v1";
    fn check(_: &u64) -> Result<(), u32> {
        Ok(())
    }
}

struct Collection;
impl ApplicationDerivedCollection<TestSchema, FlatFeature> for Collection {
    type Computation = Computation;
    type Completeness = Completeness;
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
    assert_eq!(computation.execution(), PARTITIONED);
    assert_eq!(
        computation.determinism(),
        DeterminismContract::CanonicalBitwise,
        "a computation that names no determinism contract is canonical bitwise"
    );
}

struct ProgramOf<Computation>(std::marker::PhantomData<fn() -> Computation>);
impl<Computation> ApplicationProgramDefinition<TestSchema> for ProgramOf<Computation>
where
    Computation: ApplicationManagedComputation<TestSchema, FlatFeature>,
{
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.tests.computation-partition-program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<TestSchema, FlatFeature>()
            .derived_artifact::<Artifact>()
            .managed_computation::<Computation>()
            .finish()]
    }
}

fn denial_of<Computation>() -> Option<ApplicationProgramValidationDenialKind>
where
    Computation: ApplicationManagedComputation<TestSchema, FlatFeature>,
{
    ApplicationProgramAuthoring::<TestSchema, ProgramOf<Computation>>::begin()
        .validated_program()
        .err()
        .map(|denial| denial.kind())
}

#[test]
fn execution_posture_decides_which_computation_partition_may_be_declared() {
    assert_eq!(denial_of::<Computation>(), None);
    assert_eq!(denial_of::<SerialComputation>(), None);
    let serial = ApplicationProgramAuthoring::<TestSchema, ProgramOf<SerialComputation>>::begin()
        .validated_program()
        .expect("single-kernel execution needs no resource forecast");
    assert_eq!(
        serial.features()[0].managed_computations()[0].execution(),
        ApplicationComputationExecution::Deterministic
    );
    for (case, denial) in [
        (
            "a deterministic computation with its own key",
            denial_of::<KeyedSerialComputation>(),
        ),
        (
            "a partitioned computation with the single partition",
            denial_of::<SinglePartitionedComputation>(),
        ),
    ] {
        assert_eq!(
            denial,
            Some(ApplicationProgramValidationDenialKind::MismatchedManagedComputationPartition),
            "{case}"
        );
    }
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

mod collection_binding;
