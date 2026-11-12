//! The root query requires its own current permission, independent of the edit.
use super::super::super::super::super::program as base;
use super::*;
use worth_query_decl::facade::application_program::{
    ApplicationDiscoveredActionDelivery, ApplicationDiscoveredDeliveryMode,
    ApplicationDiscoveredOutputGraph,
};
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "owned-outputs.discovered-edit.permission-root";
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
    type Discovery = query::member::Read;
    type Demand = first::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_source(
        _: &edit::Intent,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::member::Read("m".into()))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(if row.value == 5 {
            vec![first::Demand("a".into())]
        } else {
            Vec::new()
        })
    }
}
pub type Tree = ApplicationDiscoveredOutputGraph<Connection, ApplicationOutputLeaf>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationDiscoveredDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.discovered-edit.permission-delivery";
}
impl ApplicationDiscoveredActionDelivery<Schema, edit::Binding> for Delivery {
    type Root = Tree;
    type Projection = ApplicationNoProjection;
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(Tree, hierarchy::SeedTree)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.discovered-edit.permission-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = hierarchy::Program::feature_specs();
        specs[0] = base::entry_spec::<Delivery>();
        specs
    }
}
