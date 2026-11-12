//! Installed non-leaf feature/connection meaning for the generic output court.
use super::*;
pub struct Parent;
pub struct Input;
pub struct Output;
pub struct FirstSeedInput;
impl ApplicationInputPort<Schema, Parent> for FirstSeedInput {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "hierarchy-seed-input";
    const REQUIRED: bool = false;
}

impl ApplicationFeature<Schema> for Parent {
    type Inputs = ApplicationFeatureInputList<
        Input,
        ApplicationFeatureInputList<FirstSeedInput, ApplicationFeatureInputLeaf>,
    >;
    const IDENTITY: &'static str = "owned-outputs.hierarchy.parent";
}
impl ApplicationInputPort<Schema, Parent> for Input {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "hierarchy-input";
    const REQUIRED: bool = false;
}
impl ApplicationOutputPort<Schema, Parent> for Output {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "hierarchy-output";
}
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "owned-outputs.hierarchy.start";
}
impl ApplicationOccurrenceConnectionBinding<Schema, super::super::program::Entry, Parent>
    for Start
{
}
pub type RootConnection = ApplicationConnectionRef<
    Schema,
    super::super::program::Entry,
    super::super::program::EntryOutput,
    Parent,
    Input,
    Start,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = first::initial::Binding;
    type Demand = first::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &first::initial::Input,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand(input.0.key.clone()))
    }
}
pub struct ChildOutput;
impl ApplicationOutputPort<Schema, Child> for ChildOutput {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "hierarchy-child-output";
}
pub struct Child;
pub struct ChildInput;
pub struct SeedInput;
impl ApplicationFeature<Schema> for Child {
    type Inputs = ApplicationFeatureInputList<
        ChildInput,
        ApplicationFeatureInputList<SeedInput, ApplicationFeatureInputLeaf>,
    >;
    const IDENTITY: &'static str = "owned-outputs.hierarchy.child";
}
impl ApplicationInputPort<Schema, Child> for ChildInput {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "hierarchy-child-input";
    const REQUIRED: bool = false;
}
impl ApplicationInputPort<Schema, Child> for SeedInput {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "hierarchy-seed-input";
    const REQUIRED: bool = false;
}
pub struct Seed;
impl ApplicationConnectionIdentity for Seed {
    const IDENTITY: &'static str = "owned-outputs.hierarchy.seed";
}
impl ApplicationOccurrenceConnectionBinding<Schema, super::super::program::Entry, Child> for Seed {}
pub type SeedConnection = ApplicationConnectionRef<
    Schema,
    super::super::program::Entry,
    super::super::program::EntryOutput,
    Child,
    SeedInput,
    Seed,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Seed {
    type Source = second::initial::Binding;
    type Demand = second::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &second::initial::Input,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(second::Demand(input.0.key.clone()))
    }
}
pub type SeedTree = ApplicationDemandOutputGraph<SeedConnection, ApplicationOutputLeaf>;
pub struct Discover;
impl ApplicationConnectionIdentity for Discover {
    const IDENTITY: &'static str = "owned-outputs.hierarchy.discover";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Parent, Child> for Discover {}
pub type ChildConnection =
    ApplicationConnectionRef<Schema, Parent, Output, Child, ChildInput, Discover>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Discover {
    type RootDemand = first::Demand;
    type Discovery = query::first_discovery::Read;
    type Demand = second::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &first::Demand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_discovery::Read(format!("first:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<second::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        if row.value == 10 {
            return Ok(Vec::new());
        }
        Ok(vec![second::Demand(
            row.key
                .strip_prefix("first:")
                .expect("real first output key")
                .into(),
        )])
    }
}
pub type Tree = ApplicationDemandOutputGraph<
    RootConnection,
    ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>,
>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(Tree, SeedTree)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.hierarchy.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let entry = super::super::program::Program::feature_specs().remove(0);
        vec![
            entry,
            ApplicationFeatureSpec::root::<Schema, Parent>()
                .provides::<Output>()
                .conditional_operation::<first::initial::Operation>()
                .conditional_operation::<first::preserve::Operation>()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, Child>()
                .provides::<ChildOutput>()
                .conditional_operation::<from_first::initial::Operation>()
                .conditional_operation::<from_first::preserve::Operation>()
                .conditional_operation::<restricted_from_first::initial::Operation>()
                .conditional_operation::<restricted_from_first::preserve::Operation>()
                .conditional_operation::<second::initial::Operation>()
                .conditional_operation::<second::preserve::Operation>()
                .conditional_operation::<child::initial::Operation>()
                .conditional_operation::<child::preserve::Operation>()
                .conditional_operation::<restricted::initial::Operation>()
                .conditional_operation::<restricted::preserve::Operation>()
                .conditional_operation::<upstream::initial::Operation>()
                .conditional_operation::<upstream::preserve::Operation>()
                .conditional_operation::<downstream::initial::Operation>()
                .conditional_operation::<downstream::preserve::Operation>()
                .conditional_operation::<restricted_owned::initial::Operation>()
                .conditional_operation::<restricted_owned::preserve::Operation>()
                .finish(),
            super::super::predecessor::program::feature(),
            family::diamond::feature(),
            family::decision_input::feature(),
            family::parity_input::feature(),
            family::convergent::feature(),
            family::ordinary_input::feature(),
            family::decision_write::feature(),
            synchronous_edit::reconciliation::feature(),
            synchronous_edit::reconciliation::child_feature(),
        ]
    }
}
