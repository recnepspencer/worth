//! The ordinary parent's dependent changes a real member the parent consumed.
use super::super::super::program as base;
use super::*;
use family::{decision_write, ordinary_input};
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "ordinary-support.start";
}
impl ApplicationOccurrenceConnectionBinding<Schema, base::Entry, ordinary_input::Feature>
    for Start
{
}
pub type Connection = ApplicationConnectionRef<
    Schema,
    base::Entry,
    base::EntryOutput,
    ordinary_input::Feature,
    ordinary_input::InputPort,
    Start,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = edit::Binding;
    type Demand = ordinary_input::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(ordinary_input::Demand(input.key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for edit::Binding {
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<ordinary_input::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Start as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(input)
    }
}
pub struct Discover;
impl ApplicationConnectionIdentity for Discover {
    const IDENTITY: &'static str = "ordinary-support.dependent";
}
impl
    ApplicationOccurrenceConnectionBinding<Schema, ordinary_input::Feature, decision_write::Feature>
    for Discover
{
}
pub type Child = ApplicationConnectionRef<
    Schema,
    ordinary_input::Feature,
    ordinary_input::OutputPort,
    decision_write::Feature,
    decision_write::InputPort,
    Discover,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Discover {
    type RootDemand = ordinary_input::Demand;
    type Discovery = query::first_output::Read;
    type Demand = decision_write::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &ordinary_input::Demand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("first:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![decision_write::Demand(
            row.key
                .strip_prefix("first:")
                .expect("this connection reads the first-output key")
                .to_owned(),
        )])
    }
}
pub type Tree =
    ApplicationOutputGraph<Connection, ApplicationOutputEdge<Child, ApplicationOutputLeaf>>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "ordinary-support.delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for Delivery {
    type Root = Tree;
    type Projection = ApplicationNoProjection;
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("ordinary-support.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = base::Program::feature_specs();
        specs[0] = base::entry_spec::<Delivery>();
        specs
    }
}
