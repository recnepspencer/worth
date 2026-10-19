//! Absent and explicitly zero retention are different canonical policies.
use super::*;
use crate::application_program::{
    ApplicationArtifactDependency, ApplicationArtifactResourceCeiling,
    ApplicationArtifactRetention, ApplicationArtifactSuccession, ApplicationDerivedArtifact,
    ApplicationLocalityGranule, ApplicationLocalityScope,
};

struct Locality;
impl ApplicationLocalityScope for Locality {
    const IDENTITY: &'static str = "artifact-retention.locality";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Partition;
}

struct Artifact<const MODE: u8>;
impl<const MODE: u8> ApplicationDerivedArtifact<TestSchema, FlatFeature> for Artifact<MODE> {
    type Output = Export;
    type Locality = Locality;
    const IDENTITY: &'static str = "artifact-retention.output";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Retained;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Replace;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str = "artifact-retention.producer";
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] = &[];
    const REUSE_RULE: &'static str = "artifact-retention.exact-source";
    const RESOURCE_CEILING: ApplicationArtifactResourceCeiling =
        ApplicationArtifactResourceCeiling::new(
            None,
            match MODE {
                0 => None,
                1 => Some(0),
                _ => Some(4096),
            },
        );
    const STOPPED_OUTCOME: &'static str = "artifact-retention.denied";
}

struct Program<const MODE: u8>;
impl<const MODE: u8> ApplicationProgramDefinition<TestSchema> for Program<MODE> {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("artifact-retention.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<TestSchema, FlatFeature>()
            .derived_artifact::<Artifact<MODE>>()
            .finish()]
    }
}

#[test]
fn retention_absence_zero_and_finite_policy_have_distinct_program_meaning() {
    let absent = ApplicationProgramAuthoring::<TestSchema, Program<0>>::begin()
        .validated_program()
        .unwrap();
    let zero = ApplicationProgramAuthoring::<TestSchema, Program<1>>::begin()
        .validated_program()
        .unwrap();
    let finite = ApplicationProgramAuthoring::<TestSchema, Program<2>>::begin()
        .validated_program()
        .unwrap();
    assert_ne!(absent.normalized_manifest(), zero.normalized_manifest());
    assert_ne!(zero.normalized_manifest(), finite.normalized_manifest());
    assert_ne!(absent.revision(), zero.revision());
    assert_ne!(zero.revision(), finite.revision());
    assert_ne!(absent.semantic_description(), zero.semantic_description());
    assert_ne!(zero.semantic_description(), finite.semantic_description());
    assert_eq!(
        Artifact::<0>::RESOURCE_CEILING.maximum_retained_bytes(),
        None
    );
    assert_eq!(
        Artifact::<1>::RESOURCE_CEILING.maximum_retained_bytes(),
        Some(0)
    );
}
