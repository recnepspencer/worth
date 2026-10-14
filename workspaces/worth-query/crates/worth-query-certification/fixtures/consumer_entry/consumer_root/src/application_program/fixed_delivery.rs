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
    type Projection = worth_query_decl::facade::application_program::ApplicationQueryProjection<
        PlanarOutputReadBinding<ConsumerSchema>,
    >;
}
impl
    worth_query_decl::facade::application_program::ApplicationFixedQueryDelivery<
        ConsumerSchema,
        PlanarSourceAdjustmentBinding<ConsumerSchema>,
    > for ConsumerFixedDelivery
{
    type Read = PlanarOutputReadBinding<ConsumerSchema>;

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

impl
    worth_query_decl::facade::application_program::ApplicationActionDelivery<
        ConsumerSchema,
        PlanarSourceAdjustmentBinding<ConsumerSchema>,
    > for ConsumerFixedDelivery
{
    type Mode = worth_query_decl::facade::application_program::ApplicationFixedDeliveryMode<
        ConsumerProgramRoot,
        worth_query_decl::facade::application_program::ApplicationQueryProjection<
            PlanarOutputReadBinding<ConsumerSchema>,
        >,
    >;
    const IDENTITY: &'static str = "worth.query.certification.fixed-source-current-read.v1";
}
