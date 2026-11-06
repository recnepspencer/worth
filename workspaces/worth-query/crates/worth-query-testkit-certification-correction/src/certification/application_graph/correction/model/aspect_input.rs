//! Ordinary Optional parent authoring and independently disclosed required content.
use super::schema::*;
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

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    let schema = schema
        .application_query(producer_input::definition())
        .application_query_binding::<producer_input::Binding>()
        .application_query(query::definition())
        .application_query_binding::<query::Binding>()
        .application_query_inspection(super::causal_inspection::independent::metadata::<
            query::Binding,
        >("numeric-aspect-source-metadata"));
    calculation::declare(authoring::declare(schema))
}

pub use crate::facade::certification::application_graph::correction::model::schema::storage_model::{InputFacts, Bias, Sibling};
