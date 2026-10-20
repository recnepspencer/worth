use super::super::*;
use worth_query_decl::facade::application_operation::ApplicationMutationBinding;
use worth_query_decl::facade::application_program::{
    ApplicationArtifactDependency, ApplicationArtifactResourceCeiling,
    ApplicationArtifactRetention, ApplicationArtifactSuccession, ApplicationChangePosture,
    ApplicationChangeShape, ApplicationDemandOutputGraph, ApplicationDerivedArtifact,
    ApplicationLocalityGranule, ApplicationLocalityScope, ApplicationOutputGraphShape,
};
use worth_query_host::facade::application_contribution::WorthQueryProducerOutputFamily;

pub(super) type DemandRoot = ApplicationDemandOutputGraph<
    RootConnection,
    <CheckpointRoot as ApplicationOutputGraphShape<CheckpointSchema>>::Dependents,
>;
pub(super) struct DemandProgram;
pub(super) struct OptionalArtifact;
pub(super) struct SourceLocality;
struct SourceChange;
impl ApplicationLocalityScope for SourceLocality {
    const IDENTITY: &'static str = "checkpoint-optional-source";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Root;
}
impl ApplicationChangeShape for SourceChange {
    const IDENTITY: &'static str = "checkpoint-optional-source-reconstruction";
    const POSTURE: ApplicationChangePosture = ApplicationChangePosture::Reconstruct;
}
impl ApplicationDerivedArtifact<CheckpointSchema, PlanarOutputFeature> for OptionalArtifact {
    type Output = PlanarDerivedBodyOutput;
    type Locality = SourceLocality;
    const IDENTITY: &'static str = "checkpoint-optional-planar-result";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Retained;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
    const REQUIRED: bool = false;
    const PRODUCER_FAMILY: &'static str =
        <PlanarOutputFamily as WorthQueryProducerOutputFamily<CheckpointSchema>>::IDENTITY;
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] =
        &[ApplicationArtifactDependency::new(
            <PlanarSourceAdjustmentBinding<CheckpointSchema> as ApplicationMutationBinding<
                CheckpointSchema,
            >>::IDENTITY,
        )];
    const REUSE_RULE: &'static str = "exact-source";
    const RESOURCE_CEILING: ApplicationArtifactResourceCeiling =
        ApplicationArtifactResourceCeiling::new(Some(4096), Some(8192));
    const STOPPED_OUTCOME: &'static str = "optional-planar-result-stopped";
}
impl ApplicationProgramDefinition<CheckpointSchema> for DemandProgram {
    type Contributions = <CheckpointSchema as ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<DemandRoot>;
    type Rules = CheckpointRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("checkpoint-demand-only.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = CheckpointProgram::feature_specs();
        specs[0] = ApplicationFeatureSpec::root::<CheckpointSchema, PlanarSourceFeature>()
            .provides::<PlanarBodyOutput>()
            .native_action::<PlanarEditBinding<CheckpointSchema>>()
            .finish_action()
            .native_action::<PlanarSourceAdjustmentBinding<CheckpointSchema>>()
            .locality_and_change::<SourceLocality, SourceChange>()
            .finish_action()
            .finish();
        specs[1] = required_chain::output_feature_spec_with_artifact::<OptionalArtifact>();
        specs
    }
}

pub(super) struct RequiredProgram;
impl ApplicationProgramDefinition<CheckpointSchema> for RequiredProgram {
    type Contributions =
        <DemandProgram as ApplicationProgramDefinition<CheckpointSchema>>::Contributions;
    type Outputs = ApplicationProgramOutputs<CheckpointRoot>;
    type Rules = CheckpointRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("checkpoint-required-source.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        DemandProgram::feature_specs()
    }
}

pub(super) struct UngovernedProgram;
impl ApplicationProgramDefinition<CheckpointSchema> for UngovernedProgram {
    type Contributions =
        <DemandProgram as ApplicationProgramDefinition<CheckpointSchema>>::Contributions;
    type Outputs = ApplicationProgramOutputs<DemandRoot>;
    type Rules = CheckpointRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("checkpoint-ungoverned-demand.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = DemandProgram::feature_specs();
        specs[0] = ApplicationFeatureSpec::root::<CheckpointSchema, PlanarSourceFeature>()
            .provides::<PlanarBodyOutput>()
            .native_action::<PlanarEditBinding<CheckpointSchema>>()
            .finish_action()
            .native_action::<PlanarSourceAdjustmentBinding<CheckpointSchema>>()
            .finish_action()
            .finish();
        specs
    }
}
