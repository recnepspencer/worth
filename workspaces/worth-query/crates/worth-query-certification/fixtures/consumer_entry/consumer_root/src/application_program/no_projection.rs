//! Fixed required outputs explicitly delivered without a query projection.
use super::*;
use worth_query_decl::facade::application_program::{
    ApplicationActionDelivery, ApplicationActionReference, ApplicationFixedActionDelivery,
    ApplicationFixedDeliveryMode, ApplicationNoProjection,
};
pub(crate) struct NoProjectionDelivery;
impl ApplicationActionDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>>
    for NoProjectionDelivery
{
    type Mode = ApplicationFixedDeliveryMode<ConsumerProgramRoot, ApplicationNoProjection>;
    const IDENTITY: &'static str = "consumer.fixed-no-projection.v1";
}
impl ApplicationFixedActionDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>>
    for NoProjectionDelivery
{
    type Root = ConsumerProgramRoot;
    type Projection = ApplicationNoProjection;
}
pub(crate) type NoProjectionAction = ApplicationActionReference<
    ConsumerSchema,
    ApplicationRootComposition,
    PlanarSourceFeature,
    PlanarSourceAdjustmentBinding<ConsumerSchema>,
    NoProjectionDelivery,
>;
pub(crate) struct NoProjectionProgram;
impl ApplicationProgramDefinition<ConsumerSchema> for NoProjectionProgram {
    type Contributions =
        <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Contributions;
    type Outputs = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Outputs;
    type Rules = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("consumer.fixed-no-projection");
    fn adoption_authorization<V: worth_query_decl::facade::application_program::ApplicationProgramAdoptionAuthorizationVisitor<ConsumerSchema>>(visitor: V) -> V::Output{
        visitor.capability::<crate::program_deployment::ConsumerProgramDeploymentBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        features::source_delivery_specs::<NoProjectionDelivery>()
    }
}
