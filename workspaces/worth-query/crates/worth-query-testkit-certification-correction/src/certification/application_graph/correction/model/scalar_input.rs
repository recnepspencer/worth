//! Explicit scalar storage through ordinary source, producer and inspection owners.
use super::schema::*;
use worth_query_host::facade::declaration::application_schema::*;
pub mod authoring;
pub mod calculation;
pub mod continuation;
pub mod courts;
pub mod output;
pub mod program;
pub mod queries;
pub mod readiness;

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    let schema = schema
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

pub use crate::facade::certification::application_graph::correction::model::schema::storage_model::{ScalarFacts, Value, ScalarResult, Total};
