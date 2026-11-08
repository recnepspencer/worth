//! A real required producer changes the member used to discover its root.
use super::super::super::super::super::program as base;
use super::*;
use family::decision_write;
use worth_query_decl::facade::application_program::{
    ApplicationDiscoveredActionDelivery, ApplicationDiscoveredDeliveryMode,
    ApplicationDiscoveredOutputGraph,
};
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "owned-outputs.discovered-edit.support-root";
}
impl ApplicationOccurrenceConnectionBinding<Schema, base::Entry, decision_write::Feature>
    for Start
{
}
pub type Connection = ApplicationConnectionRef<
    Schema,
    base::Entry,
    base::EntryOutput,
    decision_write::Feature,
    decision_write::InputPort,
    Start,
>;
impl WorthQueryApplicationDiscoveredOutputConnection<Schema> for Start {
    type Source = edit::Binding;
    type Discovery = query::member_public::Read;
    type Demand = decision_write::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_source(
        _: &edit::Intent,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::member_public::Read("m".into()))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![decision_write::Demand(
            if row.value == 5 { "a" } else { "b" }.into(),
        )])
    }
}
pub type Tree = ApplicationDiscoveredOutputGraph<Connection, ApplicationOutputLeaf>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationDiscoveredDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.discovered-edit.support-delivery";
}
impl ApplicationDiscoveredActionDelivery<Schema, edit::Binding> for Delivery {
    type Root = Tree;
    type Projection = ApplicationNoProjection;
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.discovered-edit.support-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = hierarchy::Program::feature_specs();
        specs[0] = base::entry_spec::<Delivery>();
        specs
    }
}
