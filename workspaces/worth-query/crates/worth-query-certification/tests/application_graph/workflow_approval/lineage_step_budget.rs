//! One lineage spends one step budget. A migration successor or a fork's
//! continuation inherits every step its source took, so a loop routed through
//! a successor cannot start the budget afresh, warm or cold.

use worth_query_host::facade::application_entry::{
    WorkflowProposalPreparationDenial, WorthQueryWorkflowProposalPreparationDenial,
};
use worth_query_host::facade::declaration::application_program::ApplicationWorkflowComponentLimits;
use worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling;

use super::super::bounded_dimension_model::{
    host::publish_on_first_program,
    workflow::{
        bounded_retry_definition_with_attempts, cancel_instance, continue_on_fork,
        migrate_instance, propose_authoring_instance, retain_workflow_with_resources,
    },
};
use super::fork_continuation::fork_of;
use super::instance_cancellation::cancelled;
use super::instance_migration::{migration_denial, replace_definition, started};
use super::*;

const BUDGET: u64 = 4;
const RESUME_AT: &str = "proposal/first";

/// A running instance that has taken `steps` of its lineage's budget.
fn after_steps(
    key: u64,
    steps: u64,
) -> (
    BoundedDimensionWorkflowRuntime,
    PublishedWorkflowDefinitionRef,
    PublishedWorkflowInstanceRef,
) {
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
    let application = retain_workflow_with_resources(publish_on_first_program(), resources);
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

fn take_steps(
    application: &BoundedDimensionWorkflowRuntime,
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

pub(super) fn step_denial(
    outcome: Result<WorkflowProposalOutcome, WorthQueryWorkflowProposalPreparationDenial>,
) -> WorthQueryApplicationAttemptDenialKind {
    match outcome {
        Err(WorthQueryWorkflowProposalPreparationDenial::ProposalPreparation(
            WorkflowProposalPreparationDenial::Attempt(denial),
        )) => denial.kind(),
        other => panic!("expected a typed step denial, got {other:?}"),
    }
}

#[test]
fn a_migration_successor_spends_the_steps_its_source_took() {
    let (application, definition, instance) = after_steps(91_000, 2);
    let target = replace_definition(
        &application,
        definition,
        bounded_retry_definition_with_attempts(63),
        91_010,
    );
    let successor = started(migrate_instance(
        &application,
        instance,
        target.clone(),
        RESUME_AT,
        91_011,
    ))
    .instance()
    .clone();
    take_steps(&application, &successor, 91_020, BUDGET - 2);
    assert_eq!(
        step_denial(propose_authoring_instance(
            &application,
            successor.clone(),
            91_030
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
        "the successor's own history is short, but the lineage's budget is spent",
    );
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    assert_eq!(
        step_denial(propose_authoring_instance(
            &application,
            successor.clone(),
            91_031
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
        "a cold read spends the inherited steps too",
    );

    let next = replace_definition(
        &application,
        target,
        bounded_retry_definition_with_attempts(62),
        91_040,
    );
    assert_eq!(
        migration_denial(migrate_instance(
            &application,
            successor.clone(),
            next,
            RESUME_AT,
            91_041,
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
        "another successor cannot restart a spent budget",
    );
    let ended = cancelled(cancel_instance(&application, successor, 91_050));
    assert!(!ended.replayed(), "a spent lineage still cancels");
}

#[test]
fn a_fork_continuation_spends_the_steps_its_copy_took() {
    let (application, definition, instance) = after_steps(91_100, 3);
    let fork = fork_of(&application, instance.branch());
    let successor = started(continue_on_fork(
        &application,
        fork,
        instance.clone(),
        definition.clone(),
        RESUME_AT,
        91_110,
    ))
    .instance()
    .clone();
    take_steps(&application, &successor, 91_120, BUDGET - 3);
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    assert_eq!(
        step_denial(propose_authoring_instance(&application, successor, 91_130)),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
    );
    take_steps(&application, &instance, 91_140, BUDGET - 3);
    assert_eq!(
        step_denial(propose_authoring_instance(
            &application,
            instance.clone(),
            91_150
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
        "the source on its own branch spends its own budget independently",
    );
    let later = fork_of(&application, instance.branch());
    assert_eq!(
        migration_denial(continue_on_fork(
            &application,
            later,
            instance,
            definition,
            RESUME_AT,
            91_160,
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
        "a later fork cannot restart a spent budget",
    );
}
