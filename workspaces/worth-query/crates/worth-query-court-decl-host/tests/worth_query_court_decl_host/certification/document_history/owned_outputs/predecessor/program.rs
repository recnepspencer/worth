//! The predecessor producer is installed only into a program that declares it.
use super::super::program::{Entry, EntryOutput};
use super::*;
use family::predecessor_family as derived;
pub struct Derived;
pub struct Input;
impl ApplicationFeature<Schema> for Derived {
    type Inputs = ApplicationFeatureInputList<Input, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "owned-outputs.predecessor-derived";
}
impl ApplicationInputPort<Schema, Derived> for Input {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "predecessor-input";
    const REQUIRED: bool = false;
}
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "owned-outputs.start-predecessor";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Entry, Derived> for Start {}
pub type Connection = ApplicationConnectionRef<Schema, Entry, EntryOutput, Derived, Input, Start>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = derived::initial::Binding;
    type Demand = derived::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &derived::initial::Input,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(derived::Demand(input.0.key.clone()))
    }
}
pub type Tree = ApplicationDemandOutputGraph<Connection, ApplicationOutputLeaf>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.predecessor-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        super::super::program::Program::feature_specs()
    }
}

pub type PredecessorTree = Tree;

pub(in crate::certification::document_history::owned_outputs) fn feature() -> ApplicationFeatureSpec
{
    ApplicationFeatureSpec::root::<Schema, Derived>()
        .conditional_operation::<derived::initial::Operation>()
        .conditional_operation::<derived::preserve::Operation>()
        .finish()
}
