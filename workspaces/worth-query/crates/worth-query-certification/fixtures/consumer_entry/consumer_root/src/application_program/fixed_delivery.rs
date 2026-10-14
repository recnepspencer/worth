//! The exact root source action declares its fixed output and current read.
use super::*;
use worth_query_decl::facade::application_program::{
    ApplicationActionReference, ApplicationFixedActionDelivery,
};
use worth_query_topology_entry::{
    PlanarOutputRead, PlanarOutputReadBinding, PlanarSourceAdjustment,
};

pub struct ConsumerFixedDelivery;
impl ApplicationFixedActionDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>>
    for ConsumerFixedDelivery
{
    type Root = ConsumerProgramRoot;
    type Read = PlanarOutputReadBinding<ConsumerSchema>;
    const IDENTITY: &'static str = "worth.query.certification.fixed-source-current-read.v1";
    fn read_input(input: &PlanarSourceAdjustment) -> PlanarOutputRead {
        PlanarOutputRead {
            body_key: input.scope_key.clone(),
        }
    }
}

pub type ConsumerFixedAction = ApplicationActionReference<
    ConsumerSchema,
    ApplicationRootComposition,
    PlanarSourceFeature,
    PlanarSourceAdjustmentBinding<ConsumerSchema>,
    ConsumerFixedDelivery,
>;
