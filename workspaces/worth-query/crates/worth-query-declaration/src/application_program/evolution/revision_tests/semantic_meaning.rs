use super::{description_of, BoundedExport, BoundedFeature, RevisionSchema, SHARED_IDENTITY};
use crate::application_program::{
    ApplicationArtifactDependency, ApplicationArtifactResourceCeiling,
    ApplicationArtifactRetention, ApplicationArtifactSuccession, ApplicationComputationExecution,
    ApplicationComputationInput, ApplicationComputationPartition,
    ApplicationComputationResourceCeiling, ApplicationComputationReuse,
    ApplicationComputationStopped, ApplicationDerivedArtifact, ApplicationFeature,
    ApplicationFeatureSpec, ApplicationLocalityGranule, ApplicationLocalityScope,
    ApplicationManagedComputation, ApplicationNoOutputGraph, ApplicationProgramDefinition,
    ApplicationProgramOutputs, ApplicationRuleLeaf, ApplicationSemanticChangeKind,
    ApplicationSemanticDiff, ApplicationSemanticFamily, DeterminismContract, EquivalenceContractId,
};

struct MeaningLocality;

impl ApplicationLocalityScope for MeaningLocality {
    const IDENTITY: &'static str = "worth.query.tests.semantic-meaning-locality.v1";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Partition;
}

struct MeaningArtifact;

impl ApplicationDerivedArtifact<RevisionSchema, BoundedFeature> for MeaningArtifact {
    type Output = BoundedExport;
    type Locality = MeaningLocality;
    const IDENTITY: &'static str = "worth.query.tests.semantic-meaning-artifact.v1";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Disposable;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str = "worth.query.tests.semantic-meaning-producer.v1";
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] =
        &[ApplicationArtifactDependency::new(BoundedFeature::IDENTITY)];
    const REUSE_RULE: &'static str = "worth.query.tests.semantic-meaning-reuse.v1";
    const RESOURCE_CEILING: ApplicationArtifactResourceCeiling =
        ApplicationArtifactResourceCeiling::new(8, 4_096);
    const STOPPED_OUTCOME: &'static str = "worth.query.tests.semantic-meaning-stopped.v1";
}

struct MeaningInput;

impl ApplicationComputationInput for MeaningInput {
    type Value = u64;
    const IDENTITY: &'static str = "worth.query.tests.semantic-meaning-input.v1";
}

#[derive(Eq, Ord, PartialEq, PartialOrd, serde::Serialize)]
struct SourcePartition;
struct SourceReuse;
#[derive(Eq, Ord, PartialEq, PartialOrd, serde::Serialize)]
struct TargetPartition;
struct TargetReuse;
struct MeaningStopped;

impl ApplicationComputationPartition for SourcePartition {
    const IDENTITY: &'static str = "a|reuse=b";
}
impl ApplicationComputationReuse for SourceReuse {
    const IDENTITY: &'static str = "c";
}
impl ApplicationComputationPartition for TargetPartition {
    const IDENTITY: &'static str = "a";
}
impl ApplicationComputationReuse for TargetReuse {
    const IDENTITY: &'static str = "b|reuse=c";
}
impl ApplicationComputationStopped for MeaningStopped {
    const IDENTITY: &'static str = "worth.query.tests.semantic-meaning-stop.v1";
}

struct SourceComputation;
struct TargetComputation;
struct EquivalentComputation;
struct OtherEquivalentComputation;

macro_rules! computation {
    ($computation:ty, $partition:ty, $reuse:ty $(, $determinism:expr)?) => {
        impl ApplicationManagedComputation<RevisionSchema, BoundedFeature> for $computation {
            type Input = MeaningInput;
            type Output = MeaningArtifact;
            type Partition = $partition;
            type Reuse = $reuse;
            type Stopped = MeaningStopped;
            const IDENTITY: &'static str = "worth.query.tests.semantic-meaning-computation.v1";
            const EXECUTION: ApplicationComputationExecution =
                ApplicationComputationExecution::DeterministicPartitioned;
            $(const DETERMINISM: DeterminismContract = $determinism;)?
            const RESOURCES: ApplicationComputationResourceCeiling =
                ApplicationComputationResourceCeiling::new(16, 8_192);
        }
    };
}

computation!(SourceComputation, SourcePartition, SourceReuse);
computation!(TargetComputation, TargetPartition, TargetReuse);
computation!(
    EquivalentComputation,
    SourcePartition,
    SourceReuse,
    DeterminismContract::ContractEquivalent(EquivalenceContractId::new(7))
);
computation!(
    OtherEquivalentComputation,
    SourcePartition,
    SourceReuse,
    DeterminismContract::ContractEquivalent(EquivalenceContractId::new(8))
);

struct SourceMeaningProgram;
struct TargetMeaningProgram;
struct EquivalentMeaningProgram;
struct OtherEquivalentMeaningProgram;

macro_rules! program {
    ($program:ty, $computation:ty) => {
        impl ApplicationProgramDefinition<RevisionSchema> for $program {
            type Contributions = ();
            type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
            type Rules = ApplicationRuleLeaf;
            const IDENTITY: crate::application_program::ApplicationProgramIdentity =
                SHARED_IDENTITY;

            fn feature_specs() -> Vec<ApplicationFeatureSpec> {
                vec![
                    ApplicationFeatureSpec::root::<RevisionSchema, BoundedFeature>()
                        .derived_artifact::<MeaningArtifact>()
                        .managed_computation::<$computation>()
                        .finish(),
                ]
            }
        }
    };
}

program!(SourceMeaningProgram, SourceComputation);
program!(TargetMeaningProgram, TargetComputation);
program!(EquivalentMeaningProgram, EquivalentComputation);
program!(OtherEquivalentMeaningProgram, OtherEquivalentComputation);

#[test]
fn determinism_contract_is_authored_output_meaning() {
    let bitwise = description_of::<SourceMeaningProgram>();
    let equivalent = description_of::<EquivalentMeaningProgram>();
    let other_equivalent = description_of::<OtherEquivalentMeaningProgram>();

    for (case, source, target) in [
        ("bitwise to an equivalence contract", &bitwise, &equivalent),
        (
            "one equivalence contract to another",
            &equivalent,
            &other_equivalent,
        ),
    ] {
        assert_ne!(source.revision(), target.revision(), "{case}");
        let diff = ApplicationSemanticDiff::compare(source, target, 1_024)
            .expect("the determinism meanings fit the comparison budget");
        let changes = diff
            .changes()
            .iter()
            .map(|change| (change.family(), change.kind()))
            .collect::<Vec<_>>();
        assert_eq!(
            changes,
            [(
                ApplicationSemanticFamily::Outputs,
                ApplicationSemanticChangeKind::Changed
            )],
            "{case}"
        );
    }
}

#[test]
fn delimiter_bearing_meaning_cannot_alias_the_next_named_field() {
    let source = description_of::<SourceMeaningProgram>();
    let target = description_of::<TargetMeaningProgram>();

    assert_ne!(
        source.revision(),
        target.revision(),
        "canonical program identity must preserve the authored field boundary"
    );
    let diff = ApplicationSemanticDiff::compare(&source, &target, 1_024)
        .expect("the adversarial meanings fit the comparison budget");
    let changes = diff
        .changes()
        .iter()
        .filter(|change| change.family() == ApplicationSemanticFamily::Outputs)
        .collect::<Vec<_>>();

    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].kind(), ApplicationSemanticChangeKind::Changed);
    assert!(changes[0].migration_assessment_requirement().is_some());
    assert_ne!(changes[0].source_meaning(), changes[0].target_meaning());
}
