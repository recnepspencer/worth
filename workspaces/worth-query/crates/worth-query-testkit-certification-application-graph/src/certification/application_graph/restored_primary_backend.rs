//! A reopened application answers the patch-stream ledger position its
//! Directory Home recorded, so the first write after reopen resumes the stream
//! where close left it.

pub use worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation;
pub use worth_query_testkit_document_retention::facade::directory_home::unique_directory;
pub use worth_query_testkit_document_retention::facade::document_retention_model::host::open_directory_on_first_program;
pub use worth_relational::facade::publication::PatchStreamPosition;

pub fn ledger_position(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    installation: &WorthQueryGranularInvalidationInstallation,
) -> Option<PatchStreamPosition> {
    installation
        .retain_primary_graph_integration_handle()
        .with_runtime(request_execution, |runtime| {
            runtime.history().latest_patch_stream_position()
        })
}
