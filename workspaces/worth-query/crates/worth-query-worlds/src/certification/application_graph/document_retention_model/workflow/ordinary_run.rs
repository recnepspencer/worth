pub use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, WorthQueryApplicationRequestExt,
    WorthQueryOrdinaryWorkflowRunProgress,
};

pub use super::super::{
    host::DocumentWorkflowRuntime,
    operator_identity::{authenticate_operator, request_scope},
    retention_entry::DOCUMENT_IDENTITY,
};
pub use super::{WorkflowAdvanceInput, WorkflowAdvanceIntent};

pub fn run_instance(
    application: &DocumentWorkflowRuntime<
        impl worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition<
            super::super::schema::DocumentRetentionSchema,
        >,
    >,
    instance: PublishedWorkflowInstanceRef,
    keys: &[u64],
) -> WorthQueryOrdinaryWorkflowRunProgress {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .run_workflow(application, instance)
        .idempotency_keys(keys)
        .execute()
}
