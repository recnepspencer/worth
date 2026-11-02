use super::super::{programs::RetentionProgramP0, schema::*};
use super::ModelLifecycleBinding;
use worth_query_decl::facade::application_program::*;

pub struct ModelLifecycleFeature;
impl ApplicationFeature<DocumentRetentionSchema> for ModelLifecycleFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.certification.model-lifecycle-feature.v1";
}
pub struct ModelLifecycleProgram;
impl ApplicationProgramDefinition<DocumentRetentionSchema> for ModelLifecycleProgram {
    type Contributions = (DocumentRetentionContribution,);
    type Outputs =
        <RetentionProgramP0 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Outputs;
    type Rules =
        <RetentionProgramP0 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.model-lifecycle-program.v1");
    fn adoption_authorization<
        V: ApplicationProgramAdoptionAuthorizationVisitor<DocumentRetentionSchema>,
    >(
        visitor: V,
    ) -> V::Output {
        visitor.capability::<super::super::program_adoption::ProgramAdoptionBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut features = RetentionProgramP0::feature_specs();
        features.push(
            ApplicationFeatureSpec::root::<DocumentRetentionSchema, ModelLifecycleFeature>()
                .native_action::<ModelLifecycleBinding>()
                .finish_action()
                .finish(),
        );
        features
    }
}
pub fn validated_model_lifecycle_program(
) -> ValidatedApplicationProgram<DocumentRetentionSchema, ModelLifecycleProgram> {
    ApplicationProgramAuthoring::<DocumentRetentionSchema, ModelLifecycleProgram>::begin()
        .validated_program()
        .expect("the model lifecycle program declares its ordinary retirement/recreation action")
}
