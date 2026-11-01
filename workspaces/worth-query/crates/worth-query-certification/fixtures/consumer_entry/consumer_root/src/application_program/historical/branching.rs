//! Two actual dependent connections belong to the same initial action profile.
use super::*;
pub(crate) use crate::application_program::roots::ConsumerInitialBranchedProgramRoot as Root;

pub(crate) struct Program;
pub(crate) struct Delivery;
pub(crate) type Action = ApplicationActionReference<
    ConsumerSchema,
    InitialDiscoveredPlanarRoot,
    PlanarSourceFeature,
    Binding,
    Delivery,
>;

impl ApplicationActionDelivery<ConsumerSchema, Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Root, ApplicationNoProjection>;
    const IDENTITY: &'static str = "consumer.historical-branching.v1";
}
impl ApplicationFixedActionDelivery<ConsumerSchema, Binding> for Delivery {
    type Root = Root;
    type Projection = ApplicationNoProjection;
}

historical_program!(
    Program,
    Delivery,
    Root,
    "consumer.historical-branching-program.v1",
    [ApplicationFeatureSpec::at::<
        ConsumerSchema,
        InitialDiscoveredPlanarRoot,
        PlanarAlternateFinalOutputFeature,
    >()
    .provides::<PlanarAlternateFinalBodyOutput>()
    .conditional_operation::<PublishAlternatePlanarOutput>()
    .finish()]
);
