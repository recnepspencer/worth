//! The installed discovered root computes its set from the authored candidate.
use super::super::super::super::program as base;
use super::super::program as fixed;
use super::*;
use worth_query_decl::facade::application_program::{
    ApplicationDiscoveredActionDelivery, ApplicationDiscoveredDeliveryMode,
    ApplicationDiscoveredOutputGraph,
};
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "owned-outputs.discovered-edit.start";
}
impl ApplicationOccurrenceConnectionBinding<Schema, base::Entry, hierarchy::Parent> for Start {}
pub type Connection = ApplicationConnectionRef<
    Schema,
    base::Entry,
    base::EntryOutput,
    hierarchy::Parent,
    hierarchy::Input,
    Start,
>;
impl WorthQueryApplicationDiscoveredOutputConnection<Schema> for Start {
    type Source = edit::Binding;
    type Discovery = query::model::Read;
    type Demand = first::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_source(
        input: &edit::Intent,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::model::Read(input.key.clone()))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        if row.value == 13 {
            return Err(WorthQueryRequiredOutputConnectionDenial::new(
                "root choice thirteen is unsupported",
            ));
        }
        let mut demands = vec![first::Demand(row.key.clone())];
        if row.value >= 7 {
            demands.push(first::Demand("b".into()));
        }
        Ok(demands)
    }
}
pub type Tree = ApplicationDiscoveredOutputGraph<
    Connection,
    ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>,
>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationDiscoveredDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.discovered-edit.delivery";
}
impl ApplicationDiscoveredActionDelivery<Schema, edit::Binding> for Delivery {
    type Root = Tree;
    type Projection = ApplicationNoProjection;
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(Tree, (DemandTree, hierarchy::SeedTree))>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.discovered-edit.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = hierarchy::Program::feature_specs();
        specs[0] = base::entry_spec::<Delivery>();
        specs
    }
}

pub struct Discover;
impl ApplicationConnectionIdentity for Discover {
    const IDENTITY: &'static str = "owned-outputs.discovered-edit.child";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, hierarchy::Child>
    for Discover
{
}
pub type ChildConnection = ApplicationConnectionRef<
    Schema,
    hierarchy::Parent,
    hierarchy::Output,
    hierarchy::Child,
    hierarchy::ChildInput,
    Discover,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Discover {
    type RootDemand = first::Demand;
    type Discovery = query::first_output::Read;
    type Demand = second::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &first::Demand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        <fixed::Discover as WorthQueryApplicationDependentOutputConnection<Schema>>::discovery_from_root(root)
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        if row.value == 110 {
            return Err(WorthQueryRequiredOutputConnectionDenial::new(
                "child choice one hundred ten is unsupported",
            ));
        }
        <fixed::Discover as WorthQueryApplicationDependentOutputConnection<Schema>>::demands_from_discovery(row)
    }
}

/// A genuinely lazy read of the same already-published families.
pub type DemandTree = ApplicationDemandOutputGraph<
    ApplicationConnectionRef<
        Schema,
        base::Entry,
        base::EntryOutput,
        hierarchy::Parent,
        hierarchy::FirstSeedInput,
        hierarchy::Start,
    >,
    ApplicationOutputLeaf,
>;
