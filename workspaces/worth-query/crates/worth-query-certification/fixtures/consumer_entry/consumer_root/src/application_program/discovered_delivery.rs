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
    const IDENTITY: &'static str = "consumer.discovered-query.v2";
}
impl ApplicationDiscoveredActionDelivery<ConsumerSchema, Binding> for QueryDelivery {
    type Root = ConsumerDiscoveredProgramRoot;
    type Projection = ApplicationQueryProjection<Read>;
}
impl ApplicationDiscoveredQueryDelivery<ConsumerSchema, Binding> for QueryDelivery {
    type Read = Read;
    fn read_input(input: &PlanarSourceAdjustment) -> PlanarOutputRead {
        PlanarOutputRead {
            body_key: projected_key(input),
        }
    }
}
#[cfg(test)]
pub(crate) struct BatchDelivery<const N: usize>;
#[cfg(test)]
impl<const N: usize> ApplicationActionDelivery<ConsumerSchema, Binding> for BatchDelivery<N> {
    type Mode = ApplicationDiscoveredDeliveryMode<
        ConsumerDiscoveredProgramRoot,
        ApplicationQueryBatchProjection<Read, N>,
    >;
    const IDENTITY: &'static str = "consumer.discovered-batch.v2";
}
#[cfg(test)]
impl<const N: usize> ApplicationDiscoveredActionDelivery<ConsumerSchema, Binding>
    for BatchDelivery<N>
{
    type Root = ConsumerDiscoveredProgramRoot;
    type Projection = ApplicationQueryBatchProjection<Read, N>;
}
#[cfg(test)]
impl ApplicationDiscoveredBatchDelivery<ConsumerSchema, Binding, 2> for BatchDelivery<2> {
    type Read = Read;
    fn read_inputs(input: &PlanarSourceAdjustment) -> [PlanarOutputRead; 2] {
        [
            PlanarOutputRead {
                body_key: projected_key(input),
            },
            PlanarOutputRead {
                body_key: "sibling-b".to_owned(),
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
            .provides::<PlanarBodyOutput>().action::<Binding, D>().finish_action().finish());
        specs
    }
}

// This fixture's authored anchor discovers remote-b; the selector is pure original-input meaning.
fn projected_key(input: &PlanarSourceAdjustment) -> String {
    match input.scope_key.as_str() {
        "anchor-a" => "remote-b".to_owned(),
        other => other.to_owned(),
    }
}
