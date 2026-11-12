//! The authored edit is source-only; its actual inverse owns required reciprocal delivery.
use super::super::{edit, other_edit, program, query, schema::*};
use super::{readiness, reciprocal};
use worth_query_host::facade::{declaration::application_program::*, primary_graph::*};
pub struct Program;
pub struct Feature;
pub struct Port;
pub struct Input;
pub struct Connection;
pub struct Delivery;
pub type Root = ApplicationOutputGraph<
    ApplicationConnectionRef<
        NumericSchema,
        program::NumericFeature,
        Port,
        Feature,
        Input,
        Connection,
    >,
    ApplicationOutputLeaf,
>;
impl ApplicationFeature<NumericSchema> for Feature {
    type Inputs = ApplicationFeatureInputList<Input, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "numeric-correction.reciprocal-feature.v1";
}
impl ApplicationOutputPort<NumericSchema, program::NumericFeature> for Port {
    type Value = query::ResultBinding;
    const IDENTITY: &'static str = "numeric-correction.reciprocal-source.v1";
}
impl ApplicationInputPort<NumericSchema, Feature> for Input {
    type Value = query::ResultBinding;
    const IDENTITY: &'static str = "numeric-correction.reciprocal-input.v1";
    const REQUIRED: bool = true;
}
impl ApplicationOccurrenceConnectionBinding<NumericSchema, program::NumericFeature, Feature>
    for Connection
{
}
impl ApplicationConnectionIdentity for Connection {
    const IDENTITY: &'static str = "numeric-correction.reciprocal-connection.v1";
}
impl WorthQueryApplicationRequiredOutputConnection<NumericSchema> for Connection {
    type Source = other_edit::OtherBinding;
    type Demand = reciprocal::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        source: &other_edit::OtherEdit,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(reciprocal::Demand {
            key: source.key.clone(),
        })
    }
}
impl WorthQueryApplicationRequiredOutputSource<NumericSchema, Connection>
    for other_edit::OtherBinding
{
    fn demand_from_source(
        source: &other_edit::OtherEdit,
    ) -> Result<reciprocal::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Connection as WorthQueryApplicationRequiredOutputConnection<NumericSchema>>::demand_from_source(source)
    }
}
impl ApplicationActionDelivery<NumericSchema, other_edit::OtherBinding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Root, ApplicationNoProjection>;
    const IDENTITY: &'static str = "numeric-correction.reciprocal-delivery.v1";
}
impl ApplicationFixedActionDelivery<NumericSchema, other_edit::OtherBinding> for Delivery {
    type Root = Root;
    type Projection = ApplicationNoProjection;
}
pub fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<NumericSchema, Feature>()
        .conditional_operation::<reciprocal::CalculateReciprocal>()
        .finish()
}
impl ApplicationProgramDefinition<NumericSchema> for Program {
    type Contributions = (NumericContribution,);
    type Outputs = ApplicationProgramOutputs<Root>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("numeric-correction.deferred-reciprocal.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut features = program::model_features();
        features.push(feature());
        features.push(ApplicationFeatureSpec::root::<NumericSchema, program::NumericFeature>()
            .provides::<Port>()
            .conditional_operation::<super::super::calculation::CalculateNumber>()
            .conditional_operation::<super::super::causal_inspection::retired_outputs::calculation::CalculateRetainedNumber>()
            .conditional_operation::<super::super::causal_inspection::output_equivalence::calculation::CalculateParity>()
            .conditional_operation::<super::super::aspect_input::calculation::CalculateAspectNumber>()
            .conditional_operation::<super::super::scalar_input::calculation::CalculateScalar>()
            .action::<edit::EditBinding, program::Completion>().finish_action()
            .action::<other_edit::OtherBinding, Delivery>().finish_action()
            .action::<super::super::principal_status::Binding, program::PrincipalCompletion>().finish_action().finish());
        features
    }
}
