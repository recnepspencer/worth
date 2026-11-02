//! The application program admits native actions and real required-output demand.
use super::*;
pub type Schema = Model<0>;
pub struct Entry;
pub struct Parent;
pub struct Input;
pub struct Output;
pub struct Start;
impl<S: Scenario> ApplicationFeature<S> for Entry {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "ownership.entry.v1";
}
impl<S: Scenario> ApplicationFeature<S> for Parent {
    type Inputs = ApplicationFeatureInputList<Input, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "ownership.parent.v1";
}
impl<S: Scenario> ApplicationOutputPort<S, Entry> for Output {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "ownership.entry.output.v1";
}
impl<S: Scenario> ApplicationInputPort<S, Parent> for Input {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "ownership.parent.input.v1";
    const REQUIRED: bool = false;
}
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "ownership.start.v1";
}
impl<S: Scenario> ApplicationOccurrenceConnectionBinding<S, Entry, Parent> for Start {}
pub type Connection<S = Schema> = ApplicationConnectionRef<S, Entry, Output, Parent, Input, Start>;
impl<S: Scenario> WorthQueryApplicationRequiredOutputConnection<S> for Start {
    type Source = producer::Binding<S>;
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
impl<S: Scenario> WorthQueryApplicationOutputDemand<S> for Demand {
    type OutputFamily = producer::Family;
    fn source_intent(&self) -> query::Input {
        query::Input { id: self.0 }
    }
}
pub type Graph<S = Schema> = ApplicationDemandOutputGraph<Connection<S>, ApplicationOutputLeaf>;
pub struct Program;
impl<S: Scenario> ApplicationProgramDefinition<S> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Graph<S>>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("ownership.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<S, Entry>()
                .provides::<Output>()
                .native_action::<lifecycle::CreateBinding<S>>()
                .finish_action()
                .native_action::<lifecycle::RetireBinding<S>>()
                .finish_action()
                .native_action::<lifecycle::RestoreBinding<S>>()
                .finish_action()
                .native_action::<lifecycle::PlainBinding<S>>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::root::<S, Parent>()
                .conditional_operation::<producer::Operation>()
                .finish(),
        ]
    }
}
