//! One parent output discovers one parity producer per seeded source.
use super::super::super::super::program as base;
use super::*;
use family::{decision_input, parity_input};
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "owned-outputs.many-routes.start";
}
impl ApplicationOccurrenceConnectionBinding<Schema, base::Entry, decision_input::Feature>
    for Start
{
}
pub type Connection = ApplicationConnectionRef<
    Schema,
    base::Entry,
    base::EntryOutput,
    decision_input::Feature,
    decision_input::InputPort,
    Start,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = edit::Binding;
    type Demand = decision_input::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(decision_input::Demand(input.key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for edit::Binding {
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<decision_input::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Start as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(input)
    }
}
pub struct Discover;
impl ApplicationConnectionIdentity for Discover {
    const IDENTITY: &'static str = "owned-outputs.many-routes.discover";
}
impl ApplicationOccurrenceConnectionBinding<Schema, decision_input::Feature, parity_input::Feature>
    for Discover
{
}
pub type ChildConnection = ApplicationConnectionRef<
    Schema,
    decision_input::Feature,
    decision_input::OutputPort,
    parity_input::Feature,
    parity_input::InputPort,
    Discover,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Discover {
    type RootDemand = decision_input::Demand;
    type Discovery = query::first_output::Read;
    type Demand = parity_input::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("first:{}", root.0)))
    }
    /// The parent's value is its source value plus the seeded member value.
    /// The source value therefore names how many seeded sources to demand.
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        let routes = row.value - super::MEMBER_VALUE;
        Ok((0..routes)
            .map(|ordinal| parity_input::Demand(super::source_key(ordinal)))
            .collect())
    }
}
pub type Tree = ApplicationOutputGraph<
    Connection,
    ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>,
>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.many-routes.delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for Delivery {
    type Root = Tree;
    type Projection = ApplicationNoProjection;
}
/// The base trees stay installed so that a seeded source runs its own producers.
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(
        Tree,
        (base::FirstTree, (base::SecondTree, base::FromFirstTree)),
    )>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.many-routes.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = base::Program::feature_specs();
        specs[0] = base::entry_spec::<Delivery>();
        specs
    }
}
