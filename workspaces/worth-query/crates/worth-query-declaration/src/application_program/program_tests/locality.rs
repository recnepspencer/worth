use super::*;
use crate::application_program::{
    ApplicationArtifactDependency, ApplicationArtifactResourceCeiling,
    ApplicationArtifactRetention, ApplicationArtifactSuccession, ApplicationDerivedArtifact,
    ApplicationLocalityGranule, ApplicationLocalityScope,
};

struct TestLocality;
struct TestArtifact;

impl ApplicationLocalityScope for TestLocality {
    const IDENTITY: &'static str = "worth.query.tests.locality.partition.v1";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Partition;
}

impl ApplicationDerivedArtifact<TestSchema, FlatFeature> for TestArtifact {
    type Output = Export;
    type Locality = TestLocality;

    const IDENTITY: &'static str = "worth.query.tests.artifact.v1";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Disposable;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str = "worth.query.tests.producer.v1";
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] = &[
        ApplicationArtifactDependency::new("worth.query.tests.depth.v1"),
        ApplicationArtifactDependency::new("worth.query.tests.material.v1"),
    ];
    const REUSE_RULE: &'static str = "worth.query.tests.same-basis.v1";
    const RESOURCE_CEILING: ApplicationArtifactResourceCeiling =
        ApplicationArtifactResourceCeiling::new(8, 4096);
    const STOPPED_OUTCOME: &'static str = "worth.query.tests.artifact-stopped.v1";
}

struct ArtifactProgram;

impl ApplicationProgramDefinition<TestSchema> for ArtifactProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.tests.artifact-program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<TestSchema, FlatFeature>()
            .derived_artifact::<TestArtifact>()
            .finish()]
    }
}

#[test]
fn derived_artifact_retains_complete_installed_meaning() {
    let program = ApplicationProgramAuthoring::<TestSchema, ArtifactProgram>::begin()
        .validated_program()
        .expect("derived artifact validates");
    let feature = &program.features()[0];
    let artifact = &feature.derived_artifacts()[0];

    assert_eq!(feature.outputs()[0].identity(), Export::IDENTITY);
    assert_eq!(artifact.identity(), TestArtifact::IDENTITY);
    assert_eq!(artifact.producer_family(), TestArtifact::PRODUCER_FAMILY);
    assert_eq!(
        artifact.locality().granule(),
        ApplicationLocalityGranule::Partition
    );
    assert_eq!(artifact.dependencies(), TestArtifact::DEPENDENCIES);
    assert_eq!(artifact.resource_ceiling(), TestArtifact::RESOURCE_CEILING);
    assert_eq!(artifact.reuse_rule(), TestArtifact::REUSE_RULE);
    assert_eq!(artifact.stopped_outcome(), TestArtifact::STOPPED_OUTCOME);
}
