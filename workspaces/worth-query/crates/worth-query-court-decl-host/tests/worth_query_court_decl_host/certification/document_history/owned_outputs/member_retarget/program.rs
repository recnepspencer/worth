//! The owner's published value selects a second, distinct coupled source.
use super::super::hierarchy::program as hierarchy;
use super::super::program as entry;
use super::*;
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "member-retarget.start";
}
impl ApplicationOccurrenceConnectionBinding<Schema, entry::Entry, hierarchy::Parent> for Start {}
pub type Root = ApplicationConnectionRef<
    Schema,
    entry::Entry,
    entry::EntryOutput,
    hierarchy::Parent,
    hierarchy::Input,
    Start,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = Binding;
    type Demand = first::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand(input.owner.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for Binding {
    fn demand_from_source(
        input: &Intent,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Start::demand_from_source(input)
    }
}
pub struct Selected;
impl ApplicationConnectionIdentity for Selected {
    const IDENTITY: &'static str = "member-retarget.selected";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, hierarchy::Child>
    for Selected
{
}
pub type Connection = ApplicationConnectionRef<
    Schema,
    hierarchy::Parent,
    hierarchy::Output,
    hierarchy::Child,
    hierarchy::ChildInput,
    Selected,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Selected {
    type RootDemand = first::Demand;
    type Demand = second::Demand;
    type Discovery = query::first_output::Read;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &first::Demand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("first:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<second::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        // Only a's changed real output calls for b; b does not recurse.
        Ok(if row.key == "first:a" && row.value == 20 {
            vec![second::Demand("b".into())]
        } else {
            vec![]
        })
    }
}
pub type Tree =
    ApplicationOutputGraph<Root, ApplicationOutputEdge<Connection, ApplicationOutputLeaf>>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "member-retarget.delivery";
}
impl ApplicationFixedActionDelivery<Schema, Binding> for Delivery {
    type Root = Tree;
    type Projection = ApplicationNoProjection;
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("member-retarget.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = hierarchy::Program::feature_specs();
        specs[0] = entry::with_envelope_completion_actions(
            entry::entry_builder::<Completion, Completion, Completion>()
                .action::<Binding, Delivery>()
                .finish_action(),
        )
        .finish();
        specs
    }
}
