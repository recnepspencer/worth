//! Two real entry postures over the same owned producer and dependency graph.
use super::super::super::program::{Entry, EntryOutput};
use super::*;
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "owned-outputs.convergent.start";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Entry, producer::Feature> for Start {}
pub type Connection = ApplicationConnectionRef<
    Schema,
    Entry,
    EntryOutput,
    producer::Feature,
    producer::InputPort,
    Start,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = producer::initial::Binding;
    type Demand = producer::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &producer::initial::Input,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(producer::Demand(input.0.key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for edit::Binding {
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<producer::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(producer::Demand(input.key.clone()))
    }
}
pub type Tree = ApplicationOutputGraph<Connection, ApplicationOutputLeaf>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.convergent.delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for Delivery {
    type Root = Tree;
    type Projection = ApplicationNoProjection;
}
pub struct Deferred;
impl ApplicationProgramDefinition<Schema> for Deferred {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.convergent.deferred");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        super::super::super::program::Program::feature_specs()
    }
}
pub struct Synchronous;
impl ApplicationProgramDefinition<Schema> for Synchronous {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.convergent.synchronous");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = Deferred::feature_specs();
        specs[0] = super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}
