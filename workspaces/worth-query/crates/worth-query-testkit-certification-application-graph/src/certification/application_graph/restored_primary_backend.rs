//! A restored application answers the patch-stream ledger position its
//! checkpoint captured, so the first write after restore resumes the stream
//! where the capture left it.

pub use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;

pub use worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation;
pub use worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster;
pub use worth_relational::facade::publication::PatchStreamPosition;

pub use worth_query_testkit_document_retention::facade::document_retention_model::host::{
    publish_on_first_program, restore_on_first_program,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::programs::validated_second_program;

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
