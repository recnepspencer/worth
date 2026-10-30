//! The numeric source port feeds a separately declared required output root.
use super::super::super::{
    edit, output_program::SourcePort, program::NumericFeature, query, schema::*,
};
use super::{calculation, demand};
use worth_query_host::facade::{declaration::application_program::*, primary_graph::*};
pub struct ParityFeature;
pub struct Input;
pub struct Connection;
pub type Root = ApplicationOutputGraph<
    ApplicationConnectionRef<
        NumericSchema,
        NumericFeature,
        SourcePort,
        ParityFeature,
        Input,
        Connection,
    >,
    ApplicationOutputLeaf,
>;
impl ApplicationFeature<NumericSchema> for ParityFeature {
    type Inputs = ApplicationFeatureInputList<Input, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "parity-output.feature.v1";
}
impl ApplicationInputPort<NumericSchema, ParityFeature> for Input {
    type Value = query::ResultBinding;
    const IDENTITY: &'static str = "parity-output.input.v1";
    const REQUIRED: bool = true;
}
impl ApplicationOccurrenceConnectionBinding<NumericSchema, NumericFeature, ParityFeature>
    for Connection
{
}
impl ApplicationConnectionIdentity for Connection {
    const IDENTITY: &'static str = "parity-output.connection.v1";
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
    ApplicationFeatureSpec::root::<NumericSchema, ParityFeature>()
        .conditional_operation::<calculation::CalculateParity>()
        .finish()
}
