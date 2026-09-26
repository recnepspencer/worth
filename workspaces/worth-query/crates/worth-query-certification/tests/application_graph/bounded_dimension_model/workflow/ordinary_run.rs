use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, WorthQueryApplicationRequestExt,
    WorthQueryOrdinaryWorkflowRunProgress,
};

use super::super::{
    dimension_entry::PART_IDENTITY,
    host::BoundedDimensionWorkflowRuntime,
    operator_identity::{authenticate_operator, request_scope},
};
use super::{WorkflowAdvanceInput, WorkflowAdvanceIntent};

pub fn run_instance(
    application: &BoundedDimensionWorkflowRuntime,
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
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .run_workflow(application, instance)
        .idempotency_keys(keys)
        .execute()
}
