//! Explicit scalar storage through ordinary source, producer and inspection owners.
use super::schema::*;
use worth_query_decl::facade::{worth_query_aspect, worth_query_field};
use worth_query_host::facade::declaration::application_schema::*;
pub mod authoring;
pub mod calculation;
pub mod courts;
pub mod output;
pub mod program;
pub mod queries;
pub mod readiness;

worth_query_aspect!(pub ScalarFacts for NumericSchema, Number; identity = AspectIdentity(0x91800106), revision = AspectContractRevision(1));
worth_query_field!(pub Value for NumericSchema, Number, ScalarFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_aspect!(pub ScalarResult for NumericSchema, Number; identity = AspectIdentity(0x91800107), revision = AspectContractRevision(1));
worth_query_field!(pub Total for NumericSchema, Number, ScalarResult: u64 => U64ApplicationValueBinding, read_write, equality);

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    let schema = schema
        .optional_aspect(Number::reference(), ScalarFacts::reference())
        .scalar_aspect_field(Number::reference(), Value::reference())
        .optional_aspect(Number::reference(), ScalarResult::reference())
        .scalar_aspect_field(Number::reference(), Total::reference())
        .application_query(output::query::definition())
        .application_query_binding::<output::query::Binding>()
        .application_query_inspection(super::causal_inspection::independent::metadata::<
            output::query::Binding,
        >("numeric-scalar-output-metadata"))
        .application_query(queries::data::definition())
        .application_query_binding::<queries::data::Binding>()
        .application_query(queries::inspection::definition())
        .application_query_binding::<queries::inspection::Binding>()
        .application_query_inspection(super::causal_inspection::independent::metadata::<
            queries::inspection::Binding,
        >("numeric-scalar-source-metadata"));
    calculation::declare(authoring::declare(schema))
}
