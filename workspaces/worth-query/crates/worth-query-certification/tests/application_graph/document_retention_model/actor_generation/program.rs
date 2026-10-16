//! An isolated action program; the existing P0/P1 feature declarations stay intact.
use super::super::{programs::RetentionProgramP0, schema::*};
use super::ActorLifecycleBinding;
use worth_query_decl::facade::application_program::*;

struct ActorLifecycleFeature;
impl ApplicationFeature<DocumentRetentionSchema> for ActorLifecycleFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.certification.actor-lifecycle-feature.v1";
}
pub struct ActorProgram;
impl ApplicationProgramDefinition<DocumentRetentionSchema> for ActorProgram {
    type Contributions = (DocumentRetentionContribution,);
    type Outputs =
        <RetentionProgramP0 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Outputs;
    type Rules =
        <RetentionProgramP0 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.actor-program.v1");
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
            ApplicationFeatureSpec::root::<DocumentRetentionSchema, ActorLifecycleFeature>()
                .mutation::<ActorLifecycleBinding>()
                .finish(),
        );
        features
    }
}
pub fn validated_actor_program(
) -> ValidatedApplicationProgram<DocumentRetentionSchema, ActorProgram> {
    ApplicationProgramAuthoring::<DocumentRetentionSchema, ActorProgram>::begin()
        .validated_program()
        .expect("the isolated actor program owns the declared principal lifecycle action")
}
