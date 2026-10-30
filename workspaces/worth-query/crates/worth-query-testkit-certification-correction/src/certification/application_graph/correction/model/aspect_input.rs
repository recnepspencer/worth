//! Ordinary Optional parent authoring and independently disclosed required content.
use super::schema::*;
use worth_query_decl::facade::{worth_query_aspect, worth_query_field};
use worth_query_host::facade::declaration::application_schema::*;
pub mod authoring;
pub mod calculation;
pub mod courts;
pub mod declaration;
pub mod lifecycle;
pub mod producer_input;
pub mod program;
pub mod query;
pub mod readiness;
pub mod refusals;
worth_query_aspect!(pub InputFacts for NumericSchema, Number; identity = AspectIdentity(0x91800105), revision = AspectContractRevision(1));
worth_query_field!(pub Bias for NumericSchema, Number, InputFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub Sibling for NumericSchema, Number, InputFacts: u64 => U64ApplicationValueBinding, read_write, equality);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    let schema = schema
        .optional_aspect(Number::reference(), InputFacts::reference())
        .field(Number::reference(), Bias::reference())
        .field(Number::reference(), Sibling::reference())
        .application_query(producer_input::definition())
        .application_query_binding::<producer_input::Binding>()
        .application_query(query::definition())
        .application_query_binding::<query::Binding>()
        .application_query_inspection(super::causal_inspection::independent::metadata::<
            query::Binding,
        >("numeric-aspect-source-metadata"));
    calculation::declare(authoring::declare(schema))
}
