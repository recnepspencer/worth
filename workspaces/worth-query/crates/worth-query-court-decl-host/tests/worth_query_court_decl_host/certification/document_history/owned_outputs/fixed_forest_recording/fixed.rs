//! One fixed edit root followed by a distinct independently configured tree.
use super::*;
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "owned-outputs.fixed-forest.start";
}
impl ApplicationOccurrenceConnectionBinding<Schema, program::Entry, program::Derived> for Start {}
pub type Connection = ApplicationConnectionRef<
    Schema,
    program::Entry,
    program::EntryOutput,
    program::Derived,
    program::FirstInput,
    Start,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = edit::Binding;
    type Demand = first::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand(input.key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for edit::Binding {
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Start as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(input)
    }
}
pub type Tree = ApplicationOutputGraph<Connection, ApplicationOutputLeaf>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.fixed-forest.delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for Delivery {
    type Root = Tree;
    type Projection = ApplicationNoProjection;
}
pub type Action = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    program::Entry,
    edit::Binding,
    Delivery,
>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(Tree, program::SecondTree)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.fixed-forest.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = program::Program::feature_specs();
        specs[0] = ApplicationFeatureSpec::root::<Schema, program::Entry>()
            .provides::<program::EntryOutput>()
            .action::<synchronous_edit::new_source::Binding, Completion>()
            .finish_action()
            .action::<synchronous_edit::new_source::restricted::Binding, Completion>()
            .finish_action()
            .action::<synchronous_edit::Binding, Completion>()
            .finish_action()
            .action::<predecessor::synchronous::Binding, Completion>()
            .finish_action()
            .action::<edit::Binding, Delivery>()
            .finish_action()
            .action::<member_edit::Binding, Completion>()
            .finish_action()
            .action::<authored_member::retire::Binding, Completion>()
            .finish_action()
            .action::<authored_member::restore::Binding, Completion>()
            .finish_action()
            .action::<member_delete::Binding, Completion>()
            .finish_action()
            .finish();
        specs
    }
}
