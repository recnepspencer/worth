//! Downstream parent custody discovers a second-family child through real output data.
use super::super::program::{Child, ChildInput, Input, Output, Parent};
use super::*;
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "owned-outputs.historical-scope.start";
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
    type Source = downstream::initial::Binding;
    type Demand = downstream::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &downstream::initial::Input,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(downstream::Demand(input.0.key.clone()))
    }
}

pub struct Discover;
impl ApplicationConnectionIdentity for Discover {
    const IDENTITY: &'static str = "owned-outputs.historical-scope.discover";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Parent, Child> for Discover {}
pub type ChildConnection =
    ApplicationConnectionRef<Schema, Parent, Output, Child, ChildInput, Discover>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Discover {
    type RootDemand = downstream::Demand;
    type Discovery = query::child_output::Read;
    type Demand = second::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &downstream::Demand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::child_output::Read(format!("downstream:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        assert_eq!(row.key, "downstream:m");
        Ok(vec![second::Demand("a".into())])
    }
}
pub type Tree = ApplicationDemandOutputGraph<
    RootConnection,
    ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>,
>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.historical-scope.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let entry = super::super::super::program::Program::feature_specs().remove(0);
        vec![
            entry,
            ApplicationFeatureSpec::root::<Schema, Parent>()
                .provides::<Output>()
                .conditional_operation::<downstream::initial::Operation>()
                .conditional_operation::<downstream::preserve::Operation>()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, Child>()
                .conditional_operation::<from_first::initial::Operation>()
                .conditional_operation::<from_first::preserve::Operation>()
                .conditional_operation::<restricted_from_first::initial::Operation>()
                .conditional_operation::<restricted_from_first::preserve::Operation>()
                .conditional_operation::<second::initial::Operation>()
                .conditional_operation::<second::preserve::Operation>()
                .conditional_operation::<first::initial::Operation>()
                .conditional_operation::<first::preserve::Operation>()
                .conditional_operation::<child::initial::Operation>()
                .conditional_operation::<child::preserve::Operation>()
                .conditional_operation::<restricted::initial::Operation>()
                .conditional_operation::<restricted::preserve::Operation>()
                .conditional_operation::<upstream::initial::Operation>()
                .conditional_operation::<upstream::preserve::Operation>()
                .conditional_operation::<restricted_owned::initial::Operation>()
                .conditional_operation::<restricted_owned::preserve::Operation>()
                .finish(),
            super::super::super::predecessor::program::feature(),
            family::diamond::feature(),
            family::decision_input::feature(),
            family::ordinary_input::feature(),
            family::decision_write::feature(),
            synchronous_edit::reconciliation::feature(),
            synchronous_edit::reconciliation::child_feature(),
        ]
    }
}
