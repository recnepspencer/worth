//! Source delivery and real scalar producer share the installed ordinary program.
use super::super::schema::*;
use super::{authoring, calculation, queries::data};
use worth_query_host::facade::{declaration::application_program::*, primary_graph::*};
pub struct SourceFeature;
pub struct CalculationFeature;
pub struct Port;
pub struct Input;
pub struct Connection;
pub struct Delivery;
pub type Root = ApplicationOutputGraph<
    ApplicationConnectionRef<
        NumericSchema,
        SourceFeature,
        Port,
        CalculationFeature,
        Input,
        Connection,
    >,
    ApplicationOutputLeaf,
>;
impl ApplicationFeature<NumericSchema> for SourceFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "numeric-scalar.source-feature.v1";
}
impl ApplicationFeature<NumericSchema> for CalculationFeature {
    type Inputs = ApplicationFeatureInputList<Input, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "numeric-scalar.output-feature.v1";
}
impl ApplicationOutputPort<NumericSchema, SourceFeature> for Port {
    type Value = data::ResultBinding;
    const IDENTITY: &'static str = "numeric-scalar.source-port.v1";
}
impl ApplicationInputPort<NumericSchema, CalculationFeature> for Input {
    type Value = data::ResultBinding;
    const IDENTITY: &'static str = "numeric-scalar.output-port.v1";
    const REQUIRED: bool = true;
}
impl ApplicationOccurrenceConnectionBinding<NumericSchema, SourceFeature, CalculationFeature>
    for Connection
{
}
impl ApplicationConnectionIdentity for Connection {
    const IDENTITY: &'static str = "numeric-scalar.connection.v1";
}
impl WorthQueryApplicationRequiredOutputConnection<NumericSchema> for Connection {
    type Source = authoring::Binding;
    type Demand = calculation::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        source: &authoring::Edit,
    ) -> Result<calculation::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(calculation::Demand {
            key: source.key.clone(),
        })
    }
}
impl WorthQueryApplicationRequiredOutputSource<NumericSchema, Connection> for authoring::Binding {
    fn demand_from_source(
        source: &authoring::Edit,
    ) -> Result<calculation::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Connection as WorthQueryApplicationRequiredOutputConnection<NumericSchema>>::demand_from_source(source)
    }
}
impl ApplicationActionDelivery<NumericSchema, authoring::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Root, ApplicationNoProjection>;
    const IDENTITY: &'static str = "numeric-scalar.delivery.v1";
}
impl ApplicationFixedActionDelivery<NumericSchema, authoring::Binding> for Delivery {
    type Root = Root;
    type Projection = ApplicationNoProjection;
}
pub fn source_feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<NumericSchema, SourceFeature>()
        .provides::<Port>()
        .action::<authoring::Binding, Delivery>()
        .finish_action()
        .finish()
}
pub fn calculation_feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<NumericSchema, CalculationFeature>()
        .conditional_operation::<calculation::CalculateScalar>()
        .finish()
}
