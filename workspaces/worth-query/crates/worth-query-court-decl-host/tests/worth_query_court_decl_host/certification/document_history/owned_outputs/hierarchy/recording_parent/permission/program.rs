//! Independent authored grant facts keep the parent healthy across a denied read.
use super::super::super::program::{Child, ChildInput, Output, Parent, RootConnection};
use super::*;
pub struct Discover;
impl ApplicationConnectionIdentity for Discover {
    const IDENTITY: &'static str = "owned-outputs.recording-member-discovery";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Parent, Child> for Discover {}
pub type Connection = ApplicationConnectionRef<Schema, Parent, Output, Child, ChildInput, Discover>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Discover {
    type RootDemand = first::Demand;
    type Discovery = query::first_member_discovery::Read;
    type Demand = second::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &first::Demand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_member_discovery::Read(format!(
            "first:{}",
            root.0
        )))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![second::Demand(
            row.key
                .strip_prefix("first:")
                .expect("real parent key")
                .into(),
        )])
    }
}
pub type Tree = ApplicationDemandOutputGraph<
    RootConnection,
    ApplicationOutputEdge<Connection, ApplicationOutputLeaf>,
>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.recording-member-permission");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        <super::super::super::program::Program as ApplicationProgramDefinition<Schema>>::feature_specs()
    }
}
