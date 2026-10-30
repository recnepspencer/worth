//! Native uncertainty resolves the original key; aborted work cannot reissue its occurrence.
pub use super::{host, model, program::*};
pub use model::{operator_identity::*, retention_entry::*, schema::*};
pub use worth_query_host::facade::application_entry::*;
pub type Owner = WorthQueryWorkflowOperation<
    DocumentRetentionSchema,
    SetRetentionInput,
    WorkflowOwnerProgram,
    OccurrenceAction,
>;
pub fn original(host: &host::Host, key: u64) -> (PublishedWorkflowInstanceRef, Owner) {
    let instance = host.begin(
        super::super::workflow_execution::definition::<OccurrenceSetRetentionBinding>().unwrap(),
        8,
        key,
    );
    let required = match host.advance(instance.clone(), key + 3) {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        _ => panic!("original occurrence"),
    };
    let scope = request_scope();
    let actor = authenticate_operator(host.workflow.runtime().installed_schema(), &scope);
    let owner = host
        .workflow
        .runtime()
        .request(&actor, &scope)
        .on_branch(instance.branch())
        .mutate(SetRetentionInput {
            identity: DOCUMENT_IDENTITY.into(),
            retention_days: 8,
        })
        .without_source()
        .idempotency_owned(key + 4)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .for_workflow_operation(&host.workflow, &required)
        .unwrap_or_else(|_| panic!("original binding"))
        .retain_operation::<WorkflowOwnerProgram, OccurrenceAction>(host.workflow.program_runtime())
        .unwrap_or_else(|_| panic!("installed action"));
    (instance, owner)
}
