//! Same family/scope, distinct admitted parameters on an actual program root.
use super::super::program::{Child, ChildInput, Input, Output, Parent};
use super::*;
#[derive(Clone)]
pub struct Demand {
    pub key: String,
    pub expected: String,
}
impl WorthQueryApplicationOutputDemand<Schema> for Demand {
    type OutputFamily = first::Family;
    fn source_intent(&self) -> query::producer_model::Read {
        query::producer_model::Read {
            key: self.key.clone(),
            expected: self.expected.clone(),
        }
    }
}
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "owned-outputs.partition.start";
}
impl ApplicationOccurrenceConnectionBinding<Schema, super::super::super::program::Entry, Parent>
    for Start
{
}
pub type RootConnection = ApplicationConnectionRef<
    Schema,
    super::super::super::program::Entry,
    super::super::super::program::EntryOutput,
    Parent,
    Input,
    Start,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = first::initial::Binding;
    type Demand = Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &first::initial::Input,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand {
            key: input.0.key.clone(),
            expected: input.0.key.clone(),
        })
    }
}
pub struct Discover;
impl ApplicationConnectionIdentity for Discover {
    const IDENTITY: &'static str = "owned-outputs.partition.discover";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Parent, Child> for Discover {}
pub type Connection = ApplicationConnectionRef<Schema, Parent, Output, Child, ChildInput, Discover>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Discover {
    type RootDemand = Demand;
    type Discovery = query::first_discovery::Read;
    type Demand = second::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &Demand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_discovery::Read(format!("first:{}", root.key)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<second::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        if row.value == 10 {
            Ok(Vec::new())
        } else {
            Ok(vec![second::Demand("a".into())])
        }
    }
}
pub type Tree = ApplicationDemandOutputGraph<
    RootConnection,
    ApplicationOutputEdge<Connection, ApplicationOutputLeaf>,
>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(Tree, super::super::program::SeedTree)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.partition.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        super::super::program::Program::feature_specs()
    }
}
