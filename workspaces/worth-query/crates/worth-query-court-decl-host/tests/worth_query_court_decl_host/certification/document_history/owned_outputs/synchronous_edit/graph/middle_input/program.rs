//! A parity parent alone, and the same parent with a child that writes its member input.
use super::super::super::super::program::{entry_spec, Entry, EntryOutput};
use super::*;
use family::{decision_write, parity_input};
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "owned-outputs.middle-input.start";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Entry, parity_input::Feature> for Start {}
pub type Connection = ApplicationConnectionRef<
    Schema,
    Entry,
    EntryOutput,
    parity_input::Feature,
    parity_input::InputPort,
    Start,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = edit::Binding;
    type Demand = parity_input::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(parity_input::Demand(input.key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for edit::Binding {
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<parity_input::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Start as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(input)
    }
}
pub struct Discover;
impl ApplicationConnectionIdentity for Discover {
    const IDENTITY: &'static str = "owned-outputs.middle-input.discover";
}
impl ApplicationOccurrenceConnectionBinding<Schema, parity_input::Feature, decision_write::Feature>
    for Discover
{
}
pub type ChildConnection = ApplicationConnectionRef<
    Schema,
    parity_input::Feature,
    parity_input::OutputPort,
    decision_write::Feature,
    decision_write::InputPort,
    Discover,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Discover {
    type RootDemand = parity_input::Demand;
    type Discovery = query::first_output::Read;
    type Demand = decision_write::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("parity:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        let source = row
            .key
            .strip_prefix("parity:")
            .expect("the parity parent's own output row");
        Ok(vec![decision_write::Demand(source.to_owned())])
    }
}
pub type AloneTree = ApplicationOutputGraph<Connection, ApplicationOutputLeaf>;
pub type Tree = ApplicationOutputGraph<
    Connection,
    ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>,
>;
pub struct AloneDelivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for AloneDelivery {
    type Mode = ApplicationFixedDeliveryMode<AloneTree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.middle-input.alone-delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for AloneDelivery {
    type Root = AloneTree;
    type Projection = ApplicationNoProjection;
}
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.middle-input.delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for Delivery {
    type Root = Tree;
    type Projection = ApplicationNoProjection;
}
pub struct Alone;
impl ApplicationProgramDefinition<Schema> for Alone {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<AloneTree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.middle-input.alone");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = entry_spec::<AloneDelivery>();
        specs
    }
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.middle-input.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = entry_spec::<Delivery>();
        specs
    }
}
