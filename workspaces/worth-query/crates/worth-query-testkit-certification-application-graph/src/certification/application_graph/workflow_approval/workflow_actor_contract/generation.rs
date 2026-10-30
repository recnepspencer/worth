//! An original actor remains historical when its principal slot is genuinely reused.
pub use super::super::super::document_retention_model::{
    actor_generation::{ActorLifecycleInput, ActorLifecycleIntent, ActorProgram},
    host::publish,
    programs::{validated_first_program, validated_second_program},
    workflow::{authoring_intent, retain_workflow, support_workflow_program},
};
pub use super::*;
pub use worth_query_host::facade::{
    application_installation::WorthQueryApplicationProgramRoster,
    primary_graph::WorthQueryBranchAdoptionPublicationOutcome, runtime::ExecutionAllocationPolicy,
};

pub fn lifecycle(application: &DocumentWorkflowRuntime, restore: bool, key: u64) {
    let runtime = application.runtime();
    let scope = request_scope();
    let reviewer = authenticate_reviewer(runtime.installed_schema(), &scope);
    let outcome = runtime
        .request(&reviewer, &scope)
        .advanced()
        .mutate(ActorLifecycleIntent {
            input: ActorLifecycleInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                restore,
            },
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(
            application.program_runtime(),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "the genuine declared lifecycle action must commit: {outcome:?}"
    );
}
