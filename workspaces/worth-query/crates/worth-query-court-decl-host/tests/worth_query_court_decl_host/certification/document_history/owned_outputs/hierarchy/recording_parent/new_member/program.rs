//! A parent whose source reads only its own model discovers children through
//! the model's actual members, so a member change leaves the parent healthy.
use super::super::super::program::{Child, ChildInput, Input, Output, Parent};
use super::*;
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "owned-outputs.new-member.start";
}
impl
    ApplicationOccurrenceConnectionBinding<
        Schema,
        super::super::super::super::program::Entry,
        Parent,
    > for Start
{
}
pub type RootConnection = ApplicationConnectionRef<
    Schema,
    super::super::super::super::program::Entry,
    super::super::super::super::program::EntryOutput,
    Parent,
    Input,
    Start,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = upstream::initial::Binding;
    type Demand = upstream::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &upstream::initial::Input,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(upstream::Demand(input.0.key.clone()))
    }
}

pub struct Discover;
impl ApplicationConnectionIdentity for Discover {
    const IDENTITY: &'static str = "owned-outputs.new-member.discover";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Parent, Child> for Discover {}
pub type ChildConnection =
    ApplicationConnectionRef<Schema, Parent, Output, Child, ChildInput, Discover>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Discover {
    type RootDemand = upstream::Demand;
    type Discovery = query::members::Read;
    type Demand = downstream::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &upstream::Demand,
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
    RootConnection,
    ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>,
>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.new-member.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let entry = super::super::super::super::program::Program::feature_specs().remove(0);
        let mut features = vec![
            entry,
            ApplicationFeatureSpec::root::<Schema, Parent>()
                .provides::<Output>()
                .conditional_operation::<upstream::initial::Operation>()
                .conditional_operation::<upstream::preserve::Operation>()
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
                .conditional_operation::<downstream::initial::Operation>()
                .conditional_operation::<downstream::preserve::Operation>()
                .conditional_operation::<restricted_owned::initial::Operation>()
                .conditional_operation::<restricted_owned::preserve::Operation>()
                .finish(),
        ];
        features.extend(family::feature_specs(
            family::captured_input::program::conditional_feature(),
        ));
        features
    }
}
