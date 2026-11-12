//! The existing hierarchy's connection supplies the computed child set.
use super::*;
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "owned-outputs.synchronous-graph.start";
}
impl
    ApplicationOccurrenceConnectionBinding<
        Schema,
        super::super::super::program::Entry,
        hierarchy::Parent,
    > for Start
{
}
pub type Connection = ApplicationConnectionRef<
    Schema,
    super::super::super::program::Entry,
    super::super::super::program::EntryOutput,
    hierarchy::Parent,
    hierarchy::Input,
    Start,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = edit::Binding;
    type Demand = first::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand(input.key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for edit::Binding {
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Start as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(input)
    }
}
pub type Tree = ApplicationOutputGraph<
    Connection,
    ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>,
>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.synchronous-graph.delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for Delivery {
    type Root = Tree;
    type Projection = ApplicationNoProjection;
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(Tree, hierarchy::SeedTree)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.synchronous-graph.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = hierarchy::Program::feature_specs();
        specs[0] = super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}

/// Public READ meaning is necessary for a genuinely new parent at D.
pub struct Discover;
impl ApplicationConnectionIdentity for Discover {
    const IDENTITY: &'static str = "owned-outputs.synchronous-graph.discover";
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
        Ok(query::first_output::Read(format!("first:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<second::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        <hierarchy::Discover as WorthQueryApplicationDependentOutputConnection<Schema>>::demands_from_discovery(row)
    }
}
pub type RestrictedTree = ApplicationOutputGraph<
    Connection,
    ApplicationOutputEdge<hierarchy::ChildConnection, ApplicationOutputLeaf>,
>;
pub struct RestrictedDelivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for RestrictedDelivery {
    type Mode = ApplicationFixedDeliveryMode<RestrictedTree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.synchronous-graph.restricted-delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for RestrictedDelivery {
    type Root = RestrictedTree;
    type Projection = ApplicationNoProjection;
}
pub struct RestrictedProgram;
impl ApplicationProgramDefinition<Schema> for RestrictedProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(RestrictedTree, hierarchy::SeedTree)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.synchronous-graph.restricted-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = hierarchy::Program::feature_specs();
        specs[0] = super::super::super::program::entry_spec::<RestrictedDelivery>();
        specs
    }
}
