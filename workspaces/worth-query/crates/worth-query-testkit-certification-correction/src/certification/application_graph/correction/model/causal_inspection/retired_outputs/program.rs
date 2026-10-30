//! The numeric source port feeds a separately declared required output root.
use super::super::super::{
    edit, output_program::SourcePort, program::NumericFeature, query, schema::*,
};
use super::{calculation, demand};
use worth_query_host::facade::{declaration::application_program::*, primary_graph::*};
pub struct RetirementFeature;
pub struct Input;
pub struct Connection;
pub type Root = ApplicationOutputGraph<
    ApplicationConnectionRef<
        NumericSchema,
        NumericFeature,
        SourcePort,
        RetirementFeature,
        Input,
        Connection,
    >,
    ApplicationOutputLeaf,
>;
impl ApplicationFeature<NumericSchema> for RetirementFeature {
    type Inputs = ApplicationFeatureInputList<Input, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "retired-output.feature.v1";
}
impl ApplicationInputPort<NumericSchema, RetirementFeature> for Input {
    type Value = query::ResultBinding;
    const IDENTITY: &'static str = "retired-output.input.v1";
    const REQUIRED: bool = true;
}
impl ApplicationOccurrenceConnectionBinding<NumericSchema, NumericFeature, RetirementFeature>
    for Connection
{
}
impl ApplicationConnectionIdentity for Connection {
    const IDENTITY: &'static str = "retired-output.connection.v1";
}
impl WorthQueryApplicationRequiredOutputConnection<NumericSchema> for Connection {
    type Source = edit::EditBinding;
    type Demand = demand::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        _: &edit::Edit,
    ) -> Result<demand::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(demand::Demand)
    }
}
pub fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<NumericSchema, RetirementFeature>()
        .conditional_operation::<calculation::CalculateRetainedNumber>()
        .finish()
}
