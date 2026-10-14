//! Two lawful canonical occurrences of the same source binding.
use super::*;
use worth_query_decl::facade::application_program::{
    ApplicationActionDelivery, ApplicationActionReference, ApplicationFixedActionDelivery,
    ApplicationFixedDeliveryMode, ApplicationFixedQueryDelivery, ApplicationQueryProjection,
};
use worth_query_topology_entry::{PlanarRead, PlanarReadBinding, PlanarSourceAdjustment};

pub(crate) struct SecondarySourceDelivery;
impl ApplicationActionDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>>
    for SecondarySourceDelivery
{
    type Mode = ApplicationFixedDeliveryMode<
        ConsumerSecondaryProgramRoot,
        ApplicationQueryProjection<PlanarReadBinding<ConsumerSchema>>,
    >;
    const IDENTITY: &'static str = "consumer.secondary-source-delivery.v1";
}
impl ApplicationFixedActionDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>>
    for SecondarySourceDelivery
{
    type Root = ConsumerSecondaryProgramRoot;
    type Projection = ApplicationQueryProjection<PlanarReadBinding<ConsumerSchema>>;
}
impl ApplicationFixedQueryDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>>
    for SecondarySourceDelivery
{
    type Read = PlanarReadBinding<ConsumerSchema>;
    fn read_input(input: &PlanarSourceAdjustment) -> PlanarRead {
        PlanarRead {
            body_key: input.scope_key.clone(),
        }
    }
}
pub(crate) type SecondaryAction = ApplicationActionReference<
    ConsumerSchema,
    SecondaryPlanarRoot,
    PlanarSourceFeature,
    PlanarSourceAdjustmentBinding<ConsumerSchema>,
    SecondarySourceDelivery,
>;
pub(crate) type WrongSecondaryAction = ApplicationActionReference<
    ConsumerSchema,
    SecondaryPlanarRoot,
    PlanarSourceFeature,
    PlanarSourceAdjustmentBinding<ConsumerSchema>,
    ConsumerFixedDelivery,
>;

pub(crate) struct ActionInstanceProgram;
impl ApplicationProgramDefinition<ConsumerSchema> for ActionInstanceProgram {
    type Contributions =
        <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Contributions;
    type Outputs = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Outputs;
    type Rules = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("consumer.two-action-instances.v1");
    fn adoption_authorization<V: worth_query_decl::facade::application_program::ApplicationProgramAdoptionAuthorizationVisitor<ConsumerSchema>>(visitor: V) -> V::Output{
        visitor.capability::<crate::program_deployment::ConsumerProgramDeploymentBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut features = ConsumerProgram::feature_specs();
        let secondary = ApplicationFeatureSpec::at::<ConsumerSchema, SecondaryPlanarRoot, PlanarSourceFeature>()
            .provides::<PlanarBodyOutput>()
            .fixed_output_action::<PlanarSourceAdjustmentBinding<ConsumerSchema>, SecondarySourceDelivery>()
            .finish();
        let mut positions = features
            .iter()
            .enumerate()
            .filter(|(_, spec)| spec.feature() == secondary.feature())
            .map(|(index, _)| index);
        let index = positions
            .next()
            .expect("the actual secondary source occurrence exists");
        assert!(positions.next().is_none(), "one exact canonical occurrence");
        features[index] = secondary;
        features
    }
}
