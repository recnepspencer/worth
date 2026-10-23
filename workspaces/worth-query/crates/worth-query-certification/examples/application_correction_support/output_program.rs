//! The numeric source and its inverse share one declared required output root.
use super::{calculation, edit, other_edit, program, query, schema::*};
use worth_query_host::facade::{declaration::application_program::*, primary_graph::*};
pub struct OutputProgram;
pub struct CalculationFeature;
pub struct SourcePort;
pub struct CalculationInput;
pub struct Connection;
pub struct Delivery;
pub type Root = ApplicationOutputGraph<
    ApplicationConnectionRef<
        NumericSchema,
        program::NumericFeature,
        SourcePort,
        CalculationFeature,
        CalculationInput,
        Connection,
    >,
    ApplicationOutputLeaf,
>;
impl ApplicationFeature<NumericSchema> for CalculationFeature {
    type Inputs = ApplicationFeatureInputList<CalculationInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "numeric-correction.calculation-feature.v1";
}
impl ApplicationOutputPort<NumericSchema, program::NumericFeature> for SourcePort {
    type Value = query::ResultBinding;
    const IDENTITY: &'static str = "numeric-correction.source-port.v1";
}
impl ApplicationInputPort<NumericSchema, CalculationFeature> for CalculationInput {
    type Value = query::ResultBinding;
    const IDENTITY: &'static str = "numeric-correction.calculation-port.v1";
    const REQUIRED: bool = true;
}
impl
    ApplicationOccurrenceConnectionBinding<
        NumericSchema,
        program::NumericFeature,
        CalculationFeature,
    > for Connection
{
}
impl ApplicationConnectionIdentity for Connection {
    const IDENTITY: &'static str = "numeric-correction.required-calculation.v1";
}
impl WorthQueryApplicationRequiredOutputConnection<NumericSchema> for Connection {
    type Source = edit::EditBinding;
    type Demand = calculation::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        source: &edit::Edit,
    ) -> Result<calculation::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(calculation::Demand {
            key: source.key.clone(),
        })
    }
}
impl WorthQueryApplicationRequiredOutputSource<NumericSchema, Connection> for edit::EditBinding {
    fn demand_from_source(
        source: &edit::Edit,
    ) -> Result<calculation::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Connection as WorthQueryApplicationRequiredOutputConnection<NumericSchema>>::demand_from_source(source)
    }
}
impl WorthQueryApplicationRequiredOutputSource<NumericSchema, Connection>
    for other_edit::OtherBinding
{
    fn demand_from_source(
        source: &other_edit::OtherEdit,
    ) -> Result<calculation::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(calculation::Demand {
            key: source.key.clone(),
        })
    }
}
impl ApplicationActionDelivery<NumericSchema, edit::EditBinding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Root, ApplicationNoProjection>;
    const IDENTITY: &'static str = "numeric-correction.required-edit-delivery.v1";
}
impl ApplicationFixedActionDelivery<NumericSchema, edit::EditBinding> for Delivery {
    type Root = Root;
    type Projection = ApplicationNoProjection;
}
impl ApplicationActionDelivery<NumericSchema, other_edit::OtherBinding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Root, ApplicationNoProjection>;
    const IDENTITY: &'static str = "numeric-correction.required-inverse-delivery.v1";
}
impl ApplicationFixedActionDelivery<NumericSchema, other_edit::OtherBinding> for Delivery {
    type Root = Root;
    type Projection = ApplicationNoProjection;
}
impl ApplicationProgramDefinition<NumericSchema> for OutputProgram {
    type Contributions = (NumericContribution,);
    type Outputs = ApplicationProgramOutputs<Root>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("numeric-correction.output-program.v1");
    fn adoption_authorization<V: ApplicationProgramAdoptionAuthorizationVisitor<NumericSchema>>(
        visitor: V,
    ) -> V::Output {
        visitor.capability::<super::adoption::AdoptBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<NumericSchema, program::NumericFeature>()
                .provides::<SourcePort>()
                .action::<edit::EditBinding, Delivery>()
                .finish_action()
                .action::<other_edit::OtherBinding, Delivery>()
                .finish_action()
                .action::<super::principal_status::Binding, program::PrincipalCompletion>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::root::<NumericSchema, CalculationFeature>()
                .conditional_operation::<calculation::CalculateNumber>()
                .finish(),
        ]
    }
}
