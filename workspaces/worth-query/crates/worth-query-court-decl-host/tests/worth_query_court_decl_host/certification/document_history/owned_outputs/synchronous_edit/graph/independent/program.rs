//! A real installed fixed root over the existing independent member family.
use super::super::super::super::program as base;
use super::*;
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "owned-outputs.independent-edit.start";
}
impl ApplicationOccurrenceConnectionBinding<Schema, base::Entry, base::Derived> for Start {}
pub type Connection = ApplicationConnectionRef<
    Schema,
    base::Entry,
    base::EntryOutput,
    base::Derived,
    base::ChildInput,
    Start,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = member_edit::Binding;
    type Demand = child::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &member_edit::Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(child::Demand(input.key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for member_edit::Binding {
    fn demand_from_source(
        input: &member_edit::Intent,
    ) -> Result<child::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Start as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(input)
    }
}
pub type Tree = ApplicationOutputGraph<Connection, ApplicationOutputLeaf>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, member_edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.independent-edit.delivery";
}
impl ApplicationFixedActionDelivery<Schema, member_edit::Binding> for Delivery {
    type Root = Tree;
    type Projection = ApplicationNoProjection;
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.independent-edit.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = base::Program::feature_specs();
        specs[0] = base::entry_spec_with_member_delivery::<Completion, Delivery>();
        specs
    }
}
