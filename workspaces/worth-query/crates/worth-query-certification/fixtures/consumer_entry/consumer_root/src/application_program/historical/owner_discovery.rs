//! A separate installed fixed tree carries the owner-only discovery READ.
use super::*;
pub(crate) use crate::application_program::roots::ConsumerOwnerDiscoveryRoot as Root;

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
    const IDENTITY: &'static str = "consumer.owner-discovery-delivery.v1";
}
impl ApplicationFixedActionDelivery<ConsumerSchema, Binding> for Delivery {
    type Root = Root;
    type Projection = ApplicationNoProjection;
}
historical_program!(
    Program,
    Delivery,
    Root,
    "consumer.owner-discovery-program.v1",
    Vec::<ApplicationFeatureSpec>::new()
);
