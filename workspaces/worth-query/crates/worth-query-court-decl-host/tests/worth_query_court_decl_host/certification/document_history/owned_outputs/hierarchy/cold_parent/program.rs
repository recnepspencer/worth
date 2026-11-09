//! Same declarative ports, a new implementation discovers the actual owned members.
use super::*;
pub struct Discover;
impl ApplicationConnectionIdentity for Discover {
    const IDENTITY: &'static str = <original::Discover as ApplicationConnectionIdentity>::IDENTITY;
}
impl ApplicationOccurrenceConnectionBinding<Schema, original::Parent, original::Child>
    for Discover
{
}
pub type ChildConnection = ApplicationConnectionRef<
    Schema,
    original::Parent,
    original::Output,
    original::Child,
    original::ChildInput,
    Discover,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Discover {
    type RootDemand = first::Demand;
    type Discovery = query::members::Read;
    type Demand = downstream::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &first::Demand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::members::Read(root.0.clone()))
    }
    fn demands_from_discovery(
        row: &query::members::Members,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(row
            .members
            .iter()
            .map(|member| downstream::Demand(member.key.clone()))
            .collect())
    }
}
pub type Tree = ApplicationDemandOutputGraph<
    original::RootConnection,
    ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>,
>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(Tree, original::SeedTree)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity = original::Program::IDENTITY;
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        original::Program::feature_specs()
    }
}
