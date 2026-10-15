//! Genuine nested discovered declaration shapes over the existing Planar binding.
use super::*;
use std::marker::PhantomData;
use worth_query_decl::facade::application_program::*;
use worth_query_topology_entry::{
    PlanarOutputRead, PlanarOutputReadBinding, PlanarSourceAdjustment,
};

type Binding = PlanarSourceAdjustmentBinding<ConsumerSchema>;
type Read = PlanarOutputReadBinding<ConsumerSchema>;
pub(crate) type Action<D> = ApplicationActionReference<
    ConsumerSchema,
    DiscoveredPlanarRoot,
    PlanarSourceFeature,
    Binding,
    D,
>;
pub(crate) struct QueryDelivery;
impl ApplicationActionDelivery<ConsumerSchema, Binding> for QueryDelivery {
    type Mode = ApplicationDiscoveredDeliveryMode<
        ConsumerDiscoveredProgramRoot,
        ApplicationQueryProjection<Read>,
    >;
    const IDENTITY: &'static str = "consumer.discovered-query.v1";
}
impl ApplicationDiscoveredActionDelivery<ConsumerSchema, Binding> for QueryDelivery {
    type Root = ConsumerDiscoveredProgramRoot;
    type Projection = ApplicationQueryProjection<Read>;
}
impl ApplicationDiscoveredQueryDelivery<ConsumerSchema, Binding> for QueryDelivery {
    type Read = Read;
    fn read_input(input: &PlanarSourceAdjustment) -> PlanarOutputRead {
        PlanarOutputRead {
            body_key: input.scope_key.clone(),
        }
    }
}
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
pub(crate) struct BatchDelivery<const N: usize>;
impl<const N: usize> ApplicationActionDelivery<ConsumerSchema, Binding> for BatchDelivery<N> {
    type Mode = ApplicationDiscoveredDeliveryMode<
        ConsumerDiscoveredProgramRoot,
        ApplicationQueryBatchProjection<Read, N>,
    >;
    const IDENTITY: &'static str = "consumer.discovered-batch.v1";
}
impl<const N: usize> ApplicationDiscoveredActionDelivery<ConsumerSchema, Binding>
    for BatchDelivery<N>
{
    type Root = ConsumerDiscoveredProgramRoot;
    type Projection = ApplicationQueryBatchProjection<Read, N>;
}
impl ApplicationDiscoveredBatchDelivery<ConsumerSchema, Binding, 2> for BatchDelivery<2> {
    type Read = Read;
    fn read_inputs(input: &PlanarSourceAdjustment) -> [PlanarOutputRead; 2] {
        [
            PlanarOutputRead {
                body_key: input.scope_key.clone(),
            },
            PlanarOutputRead {
                body_key: format!("final:{}", input.scope_key),
            },
        ]
    }
}
pub(crate) struct NoProjectionDelivery;
impl ApplicationActionDelivery<ConsumerSchema, Binding> for NoProjectionDelivery {
    type Mode =
        ApplicationDiscoveredDeliveryMode<ConsumerDiscoveredProgramRoot, ApplicationNoProjection>;
    const IDENTITY: &'static str = "consumer.discovered-no-projection.v1";
}
impl ApplicationDiscoveredActionDelivery<ConsumerSchema, Binding> for NoProjectionDelivery {
    type Root = ConsumerDiscoveredProgramRoot;
    type Projection = ApplicationNoProjection;
}

pub(crate) struct Program<D>(PhantomData<fn() -> D>);
impl<D: ApplicationActionDelivery<ConsumerSchema, Binding>>
    ApplicationProgramDefinition<ConsumerSchema> for Program<D>
{
    type Contributions =
        <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Contributions;
    type Outputs = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Outputs;
    type Rules = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("consumer.discovered-contract");
    fn adoption_authorization<V: ApplicationProgramAdoptionAuthorizationVisitor<ConsumerSchema>>(
        visitor: V,
    ) -> V::Output {
        visitor.capability::<crate::program_deployment::ConsumerProgramDeploymentBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = ConsumerProgram::feature_specs();
        specs.retain(|spec| {
            !(spec.feature().composition_instance() == DiscoveredPlanarRoot::PATH
                && spec.feature().identity()
                    == <PlanarSourceFeature as ApplicationFeature<ConsumerSchema>>::IDENTITY)
        });
        specs.push(ApplicationFeatureSpec::at::<ConsumerSchema, DiscoveredPlanarRoot, PlanarSourceFeature>()
            .provides::<PlanarBodyOutput>().operation_action::<Binding, D>().finish());
        specs
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
