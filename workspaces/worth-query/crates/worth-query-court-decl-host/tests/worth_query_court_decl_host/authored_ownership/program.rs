//! The application program admits native actions and real required-output demand.
use super::*;
pub type Schema = Model<0>;
pub struct Entry;
pub struct Parent;
pub struct Input;
pub struct Output;
pub struct Start;
impl ApplicationFeature<Schema> for Entry {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "ownership.entry.v1";
}
impl ApplicationFeature<Schema> for Parent {
    type Inputs = ApplicationFeatureInputList<Input, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "ownership.parent.v1";
}
impl ApplicationOutputPort<Schema, Entry> for Output {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "ownership.entry.output.v1";
}
impl ApplicationInputPort<Schema, Parent> for Input {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "ownership.parent.input.v1";
    const REQUIRED: bool = false;
}
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "ownership.start.v1";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Entry, Parent> for Start {}
pub type Connection = ApplicationConnectionRef<Schema, Entry, Output, Parent, Input, Start>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = producer::Binding<Schema>;
    type Demand = Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &query::Input,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand(input.id))
    }
}
#[derive(Clone)]
pub struct Demand(pub u64);
impl WorthQueryApplicationOutputDemand<Schema> for Demand {
    type OutputFamily = producer::Family;
    fn source_intent(&self) -> query::Input {
        query::Input { id: self.0 }
    }
}
pub type Graph = ApplicationDemandOutputGraph<Connection, ApplicationOutputLeaf>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Graph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("ownership.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<Schema, Entry>()
                .provides::<Output>()
                .native_action::<lifecycle::CreateBinding<Schema>>()
                .finish_action()
                .native_action::<lifecycle::RetireBinding<Schema>>()
                .finish_action()
                .native_action::<lifecycle::RestoreBinding<Schema>>()
                .finish_action()
                .native_action::<lifecycle::PlainBinding<Schema>>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, Parent>()
                .conditional_operation::<producer::Operation>()
                .finish(),
        ]
    }
}
