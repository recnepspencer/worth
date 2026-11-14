//! Real corrective sources discover their remaining roots at their D position.
use super::super::super::authored_member as member;
use super::super::super::hierarchy::program as hierarchy;
use super::super::super::program as base;
use super::*;
use std::marker::PhantomData;
use worth_query_decl::facade::application_program::{
    ApplicationDiscoveredActionDelivery, ApplicationDiscoveredDeliveryMode,
    ApplicationDiscoveredOutputGraph, ApplicationDiscoveredOutputRoot,
};
#[cfg(test)]
mod courts;
#[cfg(test)]
mod selected;

macro_rules! root {
    ($start:ident, $connection:ident, $tree:ident, $port:ty, $binding:ty, $identity:literal) => {
        pub struct $start;
        impl ApplicationConnectionIdentity for $start {
            const IDENTITY: &'static str = $identity;
        }
        impl ApplicationOccurrenceConnectionBinding<Schema, base::Entry, hierarchy::Parent>
            for $start
        {
        }
        pub type $connection = ApplicationConnectionRef<
            Schema,
            base::Entry,
            base::EntryOutput,
            hierarchy::Parent,
            $port,
            $start,
        >;
        impl WorthQueryApplicationDiscoveredOutputConnection<Schema> for $start {
            type Source = $binding;
            type Discovery = query::members::Read;
            type Demand = first::Demand;
            const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
            fn discovery_from_source(
                _: &<$binding as ApplicationMutationBinding<Schema>>::Input,
            ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
                // Both existing fixture bindings are genuinely scoped to Model a.
                Ok(query::members::Read("a".into()))
            }
            fn demands_from_discovery(
                rows: &query::members::Members,
            ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
                Ok(if rows.members.is_empty() {
                    Vec::new()
                } else {
                    vec![first::Demand("a".into())]
                })
            }
        }
        pub type $tree = ApplicationDiscoveredOutputGraph<$connection, ApplicationOutputLeaf>;
    };
}
root!(
    Retire,
    RetireConnection,
    RetireTree,
    hierarchy::Input,
    member::retire::Binding,
    "owned-outputs.runtime-discovered.retire"
);
root!(
    Restore,
    RestoreConnection,
    RestoreTree,
    hierarchy::FirstSeedInput,
    member::restore::Binding,
    "owned-outputs.runtime-discovered.restore"
);

pub struct Delivery<R>(PhantomData<fn() -> R>);
impl<B, R> ApplicationActionDelivery<Schema, B> for Delivery<R>
where
    B: ApplicationMutationBinding<Schema>,
    R: ApplicationOutputGraphShape<Schema> + ApplicationDiscoveredOutputRoot,
{
    type Mode = ApplicationDiscoveredDeliveryMode<R, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.runtime-discovered.delivery";
}
impl<B, R> ApplicationDiscoveredActionDelivery<Schema, B> for Delivery<R>
where
    B: ApplicationMutationBinding<Schema>,
    R: ApplicationOutputGraphShape<Schema> + ApplicationDiscoveredOutputRoot,
{
    type Root = R;
    type Projection = ApplicationNoProjection;
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(RetireTree, RestoreTree)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.runtime-discovered");
    #[cfg(test)]
    fn adoption_authorization<V: ApplicationProgramAdoptionAuthorizationVisitor<Schema>>(
        visitor: V,
    ) -> V::Output {
        visitor.capability::<selected::Adoption>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = hierarchy::Program::feature_specs();
        specs[0] = base::with_envelope_completion_actions(base::entry_builder_with_retirement::<
            Completion,
            Completion,
            Delivery<RestoreTree>,
            Delivery<RetireTree>,
        >())
        .finish();
        specs
    }
}
