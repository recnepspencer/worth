//! Hostile counterparts reuse the real executable consumer declaration.
pub(crate) use super::discovered_delivery::{
    Action, BatchDelivery, NoProjectionDelivery, Program, QueryDelivery,
};
use super::*;
use worth_query_decl::facade::application_program::*;
use worth_query_topology_entry::{
    PlanarOutputRead, PlanarOutputReadBinding, PlanarSourceAdjustment,
};
type Binding = PlanarSourceAdjustmentBinding<ConsumerSchema>;
type Read = PlanarOutputReadBinding<ConsumerSchema>;
pub(crate) struct CounterfeitDelivery;
impl ApplicationActionDelivery<ConsumerSchema, Binding> for CounterfeitDelivery {
    type Mode = <QueryDelivery as ApplicationActionDelivery<ConsumerSchema, Binding>>::Mode;
    const IDENTITY: &'static str = QueryDelivery::IDENTITY;
}
impl ApplicationDiscoveredActionDelivery<ConsumerSchema, Binding> for CounterfeitDelivery {
    type Root = ConsumerDiscoveredProgramRoot;
    type Projection = ApplicationQueryProjection<Read>;
}
impl ApplicationDiscoveredQueryDelivery<ConsumerSchema, Binding> for CounterfeitDelivery {
    type Read = Read;
    fn read_input(_: &PlanarSourceAdjustment) -> PlanarOutputRead {
        PlanarOutputRead {
            body_key: "replacement-selector".to_owned(),
        }
    }
}
/// Deliberately attaches the nested root to the wrong real source occurrence.
pub(crate) struct WrongOccurrenceProgram;
impl ApplicationProgramDefinition<ConsumerSchema> for WrongOccurrenceProgram {
    type Contributions =
        <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Contributions;
    type Outputs = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Outputs;
    type Rules = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("consumer.discovered-wrong-occurrence");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        features::source_delivery_specs::<QueryDelivery>()
    }
}
/// Keeps the actual graph meaning but omits its discovered root from inventory.
pub(crate) struct MissingRootProgram;
impl ApplicationProgramDefinition<ConsumerSchema> for MissingRootProgram {
    type Contributions =
        <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Contributions;
    type Outputs = ApplicationProgramOutputs<(
        ConsumerProgramRoot,
        (
            ConsumerSecondaryProgramRoot,
            (
                ConsumerRequiredSharedRoot,
                ConsumerInitialDiscoveredProgramRoot,
            ),
        ),
    )>;
    type Rules = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("consumer.discovered-missing-root");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        Program::<QueryDelivery>::feature_specs()
    }
}
