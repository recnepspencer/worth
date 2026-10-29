//! One lineage spends one step budget. A migration successor or a fork's
//! continuation inherits every step its source took, so a loop routed through
//! a successor cannot start the budget afresh, warm or cold.

pub use worth_query_host::facade::application_entry::{
    WorkflowProposalPreparationDenial, WorthQueryWorkflowProposalPreparationDenial,
};
pub use worth_query_host::facade::declaration::application_program::ApplicationWorkflowComponentLimits;
pub use worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling;

pub use super::super::document_retention_model::{
    host::publish_on_first_program,
    workflow::{
        bounded_retry_definition_with_attempts, cancel_instance, continue_on_fork,
        migrate_instance, propose_authoring_instance, retain_workflow_with_resources,
    },
};
pub use super::fork_continuation::fork_of;
pub use super::instance_cancellation::cancelled;
pub use super::instance_migration::{migration_denial, replace_definition, started};
pub use super::*;

pub const BUDGET: u64 = 4;
pub const RESUME_AT: &str = "proposal/first";

/// A workflow host whose lineages each have `BUDGET` steps.
pub fn budgeted() -> DocumentWorkflowRuntime {
    let resources = WorthQueryApplicationWorkflowResourceCeiling::new(
        32,
        64,
        4,
        ApplicationWorkflowComponentLimits::new(32, 4, 128, 256, 256).unwrap(),
        64 * 1024,
        32,
        u32::try_from(BUDGET).unwrap(),
        256 * 1024,
    )
    .expect("the workflow installation limits are nonzero");
    retain_workflow_with_resources(publish_on_first_program(), resources)
}

/// A running instance that has taken `steps` of its lineage's budget.
pub fn after_steps(
    key: u64,
    steps: u64,
) -> (
    DocumentWorkflowRuntime,
    PublishedWorkflowDefinitionRef,
    PublishedWorkflowInstanceRef,
) {
    let application = budgeted();
    let definition = match publish_definition(
        &application,
        bounded_retry_definition_with_attempts(64),
        WorkflowDefinitionExpectedPredecessor::Absent,
        key,
    )
    .expect("the retry definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("the retry definition did not publish: {other:?}"),
    };
    let instance = match start_instance(&application, definition.clone(), key + 1)
        .expect("the instance start prepares")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("the instance did not start: {other:?}"),
    };
    take_steps(&application, &instance, key + 2, steps);
    (application, definition, instance)
}

pub fn take_steps(
    application: &DocumentWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    key: u64,
    steps: u64,
) {
    for offset in 0..steps {
        match propose_authoring_instance(application, instance.clone(), key + offset) {
            Ok(WorkflowProposalOutcome::Published(_)) => {}
            other => panic!("step {offset} of the budget did not publish: {other:?}"),
        }
    }
}

pub fn step_denial(
    outcome: Result<WorkflowProposalOutcome, WorthQueryWorkflowProposalPreparationDenial>,
) -> WorthQueryApplicationAttemptDenialKind {
    match outcome {
        Err(WorthQueryWorkflowProposalPreparationDenial::ProposalPreparation(
            WorkflowProposalPreparationDenial::Attempt(denial),
        )) => denial.kind(),
        other => panic!("expected a typed step denial, got {other:?}"),
    }
}
