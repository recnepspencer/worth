//! Source-owned association around the sole funded query-parameter capture.
use super::OriginalSourceDisclosure;
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial;
use worth_query_declaration::facade::{
    application_query::{ApplicationQueryBinding, ApplicationQueryParameterSet},
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

pub fn capture_parameters<Schema, Binding>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
    framework_bytes: usize,
    project: impl FnOnce() -> Result<
        ApplicationQueryParameterSet<Binding::Query>,
        WorthQueryApplicationAttemptDenial,
    >,
) -> Result<OriginalSourceDisclosure, WorthQueryApplicationAttemptDenial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationQueryBinding<Schema>,
{
    crate::domain_computation::primary_graph::application_query::capture_parameters::<
        Schema,
        Binding,
    >(execution, installed_schema, framework_bytes, project)
    .map(OriginalSourceDisclosure::captured)
}
