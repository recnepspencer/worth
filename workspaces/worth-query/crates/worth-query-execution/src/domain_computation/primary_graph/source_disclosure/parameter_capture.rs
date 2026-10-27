//! Source-owned association around the sole funded query-parameter capture.
use super::OriginalSourceDisclosure;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_declaration::facade::{
    application_query::{ApplicationQueryBinding, ApplicationQueryParameterSet},
    application_schema::ApplicationSchema,
};

pub(in crate::domain_computation::primary_graph) fn capture_parameters<Schema, Binding>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
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
    super::super::application_query::capture_parameters::<Schema, Binding>(
        execution,
        runtime,
        framework_bytes,
        project,
    )
    .map(OriginalSourceDisclosure::captured)
}
