//! Installed homogeneous fixed batch over two original-input output selectors.
use super::*;
use worth_query_decl::facade::application_program::{
    ApplicationActionDelivery, ApplicationActionReference, ApplicationFixedActionDelivery,
    ApplicationFixedBatchDelivery, ApplicationFixedDeliveryMode, ApplicationQueryBatchProjection,
};
pub(crate) struct BatchDelivery;
impl ApplicationActionDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>>
    for BatchDelivery
{
    type Mode = ApplicationFixedDeliveryMode<
        ConsumerProgramRoot,
        ApplicationQueryBatchProjection<
            worth_query_topology_entry::PlanarOutputReadBinding<ConsumerSchema>,
            2,
        >,
    >;
    const IDENTITY: &'static str = "consumer.fixed-batch.v1";
}
impl ApplicationFixedActionDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>>
    for BatchDelivery
{
    type Root = ConsumerProgramRoot;
    type Projection = ApplicationQueryBatchProjection<
        worth_query_topology_entry::PlanarOutputReadBinding<ConsumerSchema>,
        2,
    >;
}
pub(crate) type BatchAction = ApplicationActionReference<
    ConsumerSchema,
    ApplicationRootComposition,
    PlanarSourceFeature,
    PlanarSourceAdjustmentBinding<ConsumerSchema>,
    BatchDelivery,
>;
pub(crate) struct BatchProgram;
impl ApplicationProgramDefinition<ConsumerSchema> for BatchProgram {
    type Contributions =
        <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Contributions;
    type Outputs = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Outputs;
    type Rules = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("consumer.fixed-batch");
    fn adoption_authorization<V: worth_query_decl::facade::application_program::ApplicationProgramAdoptionAuthorizationVisitor<ConsumerSchema>>(visitor: V) -> V::Output{
        visitor.capability::<crate::program_deployment::ConsumerProgramDeploymentBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        features::source_delivery_specs::<BatchDelivery>()
    }
}

impl ApplicationFixedBatchDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>, 2>
    for BatchDelivery
{
    type Read = worth_query_topology_entry::PlanarOutputReadBinding<ConsumerSchema>;
    fn read_inputs(
        input: &worth_query_topology_entry::PlanarSourceAdjustment,
    ) -> [worth_query_topology_entry::PlanarOutputRead; 2] {
        [
            worth_query_topology_entry::PlanarOutputRead {
                body_key: input.scope_key.clone(),
            },
            worth_query_topology_entry::PlanarOutputRead {
                body_key: format!("final:{}", input.scope_key),
            },
        ]
    }
}

pub(crate) struct CounterfeitBatchDelivery;
impl ApplicationActionDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>>
    for CounterfeitBatchDelivery
{
    type Mode = <BatchDelivery as ApplicationActionDelivery<
        ConsumerSchema,
        PlanarSourceAdjustmentBinding<ConsumerSchema>,
    >>::Mode;
    const IDENTITY: &'static str = <BatchDelivery as ApplicationActionDelivery<
        ConsumerSchema,
        PlanarSourceAdjustmentBinding<ConsumerSchema>,
    >>::IDENTITY;
}
impl ApplicationFixedActionDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>>
    for CounterfeitBatchDelivery
{
    type Root = ConsumerProgramRoot;
    type Projection = <BatchDelivery as ApplicationFixedActionDelivery<
        ConsumerSchema,
        PlanarSourceAdjustmentBinding<ConsumerSchema>,
    >>::Projection;
}
impl ApplicationFixedBatchDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>, 2>
    for CounterfeitBatchDelivery
{
    type Read = worth_query_topology_entry::PlanarOutputReadBinding<ConsumerSchema>;
    fn read_inputs(
        _: &worth_query_topology_entry::PlanarSourceAdjustment,
    ) -> [worth_query_topology_entry::PlanarOutputRead; 2] {
        [
            worth_query_topology_entry::PlanarOutputRead {
                body_key: "remote-a".to_owned(),
            },
            worth_query_topology_entry::PlanarOutputRead {
                body_key: "remote-a".to_owned(),
            },
        ]
    }
}
pub(crate) type CounterfeitBatchAction = ApplicationActionReference<
    ConsumerSchema,
    ApplicationRootComposition,
    PlanarSourceFeature,
    PlanarSourceAdjustmentBinding<ConsumerSchema>,
    CounterfeitBatchDelivery,
>;

pub(crate) struct EmptyBatchDelivery;
impl ApplicationActionDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>>
    for EmptyBatchDelivery
{
    type Mode = ApplicationFixedDeliveryMode<
        ConsumerProgramRoot,
        ApplicationQueryBatchProjection<
            worth_query_topology_entry::PlanarOutputReadBinding<ConsumerSchema>,
            0,
        >,
    >;
    const IDENTITY: &'static str = "consumer.empty-batch.v1";
}
impl ApplicationFixedActionDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>>
    for EmptyBatchDelivery
{
    type Root = ConsumerProgramRoot;
    type Projection = ApplicationQueryBatchProjection<
        worth_query_topology_entry::PlanarOutputReadBinding<ConsumerSchema>,
        0,
    >;
}
pub(crate) struct EmptyBatchProgram;
impl ApplicationProgramDefinition<ConsumerSchema> for EmptyBatchProgram {
    type Contributions =
        <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Contributions;
    type Outputs = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Outputs;
    type Rules = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("consumer.empty-batch");
    fn adoption_authorization<V: worth_query_decl::facade::application_program::ApplicationProgramAdoptionAuthorizationVisitor<ConsumerSchema>>(visitor: V) -> V::Output{
        visitor.capability::<crate::program_deployment::ConsumerProgramDeploymentBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        features::source_delivery_specs::<EmptyBatchDelivery>()
    }
}

pub(crate) struct TripleBatchDelivery;
impl ApplicationActionDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>>
    for TripleBatchDelivery
{
    type Mode = ApplicationFixedDeliveryMode<
        ConsumerProgramRoot,
        ApplicationQueryBatchProjection<
            worth_query_topology_entry::PlanarOutputReadBinding<ConsumerSchema>,
            3,
        >,
    >;
    const IDENTITY: &'static str = <BatchDelivery as ApplicationActionDelivery<
        ConsumerSchema,
        PlanarSourceAdjustmentBinding<ConsumerSchema>,
    >>::IDENTITY;
}
impl ApplicationFixedActionDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>>
    for TripleBatchDelivery
{
    type Root = ConsumerProgramRoot;
    type Projection = ApplicationQueryBatchProjection<
        worth_query_topology_entry::PlanarOutputReadBinding<ConsumerSchema>,
        3,
    >;
}
impl ApplicationFixedBatchDelivery<ConsumerSchema, PlanarSourceAdjustmentBinding<ConsumerSchema>, 3>
    for TripleBatchDelivery
{
    type Read = worth_query_topology_entry::PlanarOutputReadBinding<ConsumerSchema>;
    fn read_inputs(
        input: &worth_query_topology_entry::PlanarSourceAdjustment,
    ) -> [worth_query_topology_entry::PlanarOutputRead; 3] {
        let [first, second] = BatchDelivery::read_inputs(input);
        [
            first,
            second,
            worth_query_topology_entry::PlanarOutputRead {
                body_key: input.scope_key.clone(),
            },
        ]
    }
}
pub(crate) struct TripleBatchProgram;
impl ApplicationProgramDefinition<ConsumerSchema> for TripleBatchProgram {
    type Contributions =
        <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Contributions;
    type Outputs = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Outputs;
    type Rules = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        <BatchProgram as ApplicationProgramDefinition<ConsumerSchema>>::IDENTITY;
    fn adoption_authorization<V: worth_query_decl::facade::application_program::ApplicationProgramAdoptionAuthorizationVisitor<ConsumerSchema>>(visitor: V) -> V::Output{
        visitor.capability::<crate::program_deployment::ConsumerProgramDeploymentBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        features::source_delivery_specs::<TripleBatchDelivery>()
    }
}
