#![forbid(unsafe_code)]
mod contribution;

use worth_query_consumer_values::PositiveCount;
use worth_query_decl::facade::{
    application_schema::ApplicationSchema, worth_query_application_contribution,
    worth_query_aspect, worth_query_entity, worth_query_field, worth_query_unit,
    worth_query_value_binding,
};

pub trait ParameterSchemaBinding: ApplicationSchema {}

worth_query_value_binding! {
    pub ParameterCountBinding for PositiveCount {
        identity: "worth.query.certification.parameter-count.v1",
        scalar: UInt64,
        unit: "CountUnit",
        encode: PositiveCount::get,
        decode: PositiveCount::new,
    }
}

worth_query_entity!(pub Parameter for Schema: ParameterSchemaBinding);
worth_query_unit!(pub CountUnit(()) for Schema: ParameterSchemaBinding);
worth_query_aspect!(
    pub ParameterValue for Schema: ParameterSchemaBinding, Parameter;
    identity = AspectIdentity(0x9174_1002),
    revision = AspectContractRevision(1),
);
worth_query_field!(
    pub Count for Schema: ParameterSchemaBinding, Parameter, ParameterValue:
    PositiveCount => ParameterCountBinding, unit CountUnit, read_write, equality
);

worth_query_application_contribution! {
    pub contribution ParameterContribution for Schema: ParameterSchemaBinding {
        identity: "worth.query.certification.parameter.v1",
        members: |schema| {
            schema
                .entity(Parameter::reference::<Schema>())
                .unit(CountUnit::reference::<Schema>())
                .aspect(
                    Parameter::reference::<Schema>(),
                    ParameterValue::reference::<Schema>(),
                )
                .field(Parameter::reference::<Schema>(), Count::reference::<Schema>())
        }
    }
}
