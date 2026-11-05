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
        vec![entry_spec::<S>(), {
            let spec = ApplicationFeatureSpec::root::<S, Parent>()
                .conditional_operation::<producer::Operation>();
            let spec = if S::COUPLED {
                spec.conditional_operation::<coupled::producer::Operation>()
            } else {
                spec
            };
            spec.finish()
        }]
    }
}

pub(super) fn entry_spec<S: Scenario>() -> ApplicationFeatureSpec {
    let spec = ApplicationFeatureSpec::root::<S, Entry>()
        .provides::<Output>()
        .native_action::<lifecycle::CreateBinding<S>>()
        .finish_action()
        .native_action::<lifecycle::RetireBinding<S>>()
        .finish_action()
        .native_action::<lifecycle::RestoreBinding<S>>()
        .finish_action()
        .native_action::<lifecycle::PlainBinding<S>>()
        .finish_action();
    if matches!(S::MODE, 30 | 39 | 40 | 41 | 60) {
        spec.native_action::<relation_splice::inspection::Binding<S>>()
            .finish_action()
            .finish()
    } else if matches!(S::MODE, 68 | 69 | 70) {
        let spec = if S::MODE == 70 {
            spec.native_action::<foreign_aggregate::BirthBinding<S>>()
                .finish_action()
                .native_action::<sheet_boundary::split_walk::ExtendBinding<S>>()
                .finish_action()
        } else {
            spec
        };
        spec.action::<parameter_inverse::Binding<S>, super::cold_owned_revision::Delivery>()
            .finish_action()
            .finish()
    } else if S::MODE == 48 {
        spec.native_action::<parameter_inverse::Binding<S>>()
            .finish_action()
            .finish()
    } else if S::MODE == 29 {
        spec.native_action::<projected_memberships::inspection::Binding<S>>()
            .finish_action()
            .finish()
    } else {
        spec.finish()
    }
}
