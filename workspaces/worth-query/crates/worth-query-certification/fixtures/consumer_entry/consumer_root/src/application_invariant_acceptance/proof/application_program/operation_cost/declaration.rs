//! Separate installed cost action; the baseline ConsumerProgram/P0 remains unchanged.
use super::*;
pub(super) struct Delivery;
impl ApplicationActionDelivery<Schema, PlanarSourceAdjustmentBinding<Schema>> for Delivery {
    type Mode = ApplicationDiscoveredDeliveryMode<
        FanoutRoot,
        ApplicationQueryProjection<PlanarOutputReadBinding<Schema>>,
    >;
    const IDENTITY: &'static str = "membership.cost.original-target.v1";
}
impl ApplicationDiscoveredActionDelivery<Schema, PlanarSourceAdjustmentBinding<Schema>>
    for Delivery
{
    type Root = FanoutRoot;
    type Projection = ApplicationQueryProjection<PlanarOutputReadBinding<Schema>>;
}
impl ApplicationDiscoveredQueryDelivery<Schema, PlanarSourceAdjustmentBinding<Schema>>
    for Delivery
{
    type Read = PlanarOutputReadBinding<Schema>;
    fn read_input(input: &PlanarSourceAdjustment) -> PlanarOutputRead {
        PlanarOutputRead {
            body_key: format!("{}-target-0-a", input.scope_key),
        }
    }
}
pub(super) type Action = ApplicationActionReference<
    Schema,
    Fanout,
    PlanarSourceFeature,
    PlanarSourceAdjustmentBinding<Schema>,
    Delivery,
>;
pub(super) struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = <MembershipProgram as ApplicationProgramDefinition<Schema>>::Contributions;
    type Outputs = <MembershipProgram as ApplicationProgramDefinition<Schema>>::Outputs;
    type Rules = <MembershipProgram as ApplicationProgramDefinition<Schema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("membership.discovered-cost.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = MembershipProgram::feature_specs();
        specs.retain(|spec| {
            !(spec.feature().composition_instance() == Fanout::PATH
                && spec.feature().identity()
                    == <PlanarSourceFeature as ApplicationFeature<Schema>>::IDENTITY)
        });
        specs.push(
            ApplicationFeatureSpec::at::<Schema, Fanout, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .operation_action::<PlanarSourceAdjustmentBinding<Schema>, Delivery>()
                .finish(),
        );
        specs
    }
}
