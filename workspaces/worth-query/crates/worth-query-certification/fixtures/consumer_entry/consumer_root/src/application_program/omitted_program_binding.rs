use worth_query_decl::facade::application_program::{
    ApplicationFeature, ApplicationFeatureInputLeaf, ApplicationFeatureSpec,
    ApplicationNoOutputGraph, ApplicationProgramAuthoring, ApplicationProgramDefinition,
    ApplicationProgramIdentity, ApplicationProgramOutputs, ValidatedApplicationProgram,
};
use worth_query_parameter_entry::ParameterFeature;
use worth_query_topology_entry::{
    MutatePlanar, PublishAlternatePlanarOutput, PublishFinalPlanarOutput,
};

use super::{ConsumerRules, ConsumerSchema};

pub(crate) struct OmittedProgramBinding;
struct InstalledConditionalActionFeature;

impl ApplicationFeature<ConsumerSchema> for InstalledConditionalActionFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.certification.installed-conditional-action-feature";
}

impl ApplicationProgramDefinition<ConsumerSchema> for OmittedProgramBinding {
    type Contributions = <ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ConsumerRules;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.omitted-program-binding.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<ConsumerSchema, InstalledConditionalActionFeature>()
                .conditional_operation::<MutatePlanar>()
                .conditional_operation::<PublishFinalPlanarOutput>()
                .conditional_operation::<PublishAlternatePlanarOutput>()
                .finish(),
            ApplicationFeatureSpec::root::<ConsumerSchema, ParameterFeature>().finish(),
        ]
    }
}

pub(crate) fn validated_omitted_program_binding() -> ValidatedApplicationProgram<
    ConsumerSchema,
    OmittedProgramBinding,
> {
    ApplicationProgramAuthoring::<ConsumerSchema, OmittedProgramBinding>::begin()
        .validated_program()
        .expect("the omitted-binding program is declaration-valid")
}
