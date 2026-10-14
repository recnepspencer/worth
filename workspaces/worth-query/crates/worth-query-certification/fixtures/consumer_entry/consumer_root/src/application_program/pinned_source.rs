//! Isolated receipt-only action; the existing consumer/P0 canonical meaning is unchanged.
use super::*;
use worth_query_decl::facade::application_program::{
    ApplicationActionDelivery, ApplicationActionReference, ApplicationNoOutputGraph,
    ApplicationSourceCompletion,
};
pub(crate) struct PinnedCompletion;
impl ApplicationActionDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>>
    for PinnedCompletion
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "consumer.pinned-source-completion.v1";
}
pub(crate) type PinnedAction = ApplicationActionReference<
    ConsumerSchema,
    ApplicationRootComposition,
    PlanarSourceFeature,
    PlanarSourceAdjustmentBinding<ConsumerSchema>,
    PinnedCompletion,
>;
pub(crate) struct PinnedSourceProgram;
impl ApplicationProgramDefinition<ConsumerSchema> for PinnedSourceProgram {
    type Contributions =
        <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Contributions;
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("consumer.pinned-source.v1");
    fn adoption_authorization<V: worth_query_decl::facade::application_program::ApplicationProgramAdoptionAuthorizationVisitor<ConsumerSchema>>(visitor: V) -> V::Output{
        visitor.capability::<crate::program_deployment::ConsumerProgramDeploymentBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<ConsumerSchema, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .repeated_optional_member_with_external_input::<PlanarEditBinding<ConsumerSchema>,
                    correspondence::OptionalAdjustmentCorrespondence, external_input::NeutralExternalProvider>()
                .mutation::<PriorCycleAdjustmentBinding<ConsumerSchema>>()
                .mutation_with_external_input::<VertexReplacementBinding<ConsumerSchema>, external_input::NeutralExternalProvider>()
                .conditional_operation::<MutatePlanar>()
                .conditional_operation::<PublishFinalPlanarOutput>()
                .conditional_operation::<PreserveFinalPlanarOutput>()
                .conditional_operation::<PublishAlternatePlanarOutput>()
                .operation_action::<PlanarSourceAdjustmentBinding<ConsumerSchema>, PinnedCompletion>()
                .finish(),
            ApplicationFeatureSpec::root::<ConsumerSchema, ParameterFeature>().finish(),
        ]
    }
}
