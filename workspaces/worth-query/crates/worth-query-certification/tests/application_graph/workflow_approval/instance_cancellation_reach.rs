//! What one cancellation reaches: only the copy of the instance on the
//! request's own branch, every effect a migrated instance carried from its
//! source, and nothing at all without the start capability.

use worth_query_host::facade::application_entry::{
    WorkflowInstancePreparationDenial, WorthQueryWorkflowInstanceStartPreparationDenial,
};

use super::super::bounded_dimension_model::workflow::{
    cancel_instance, continue_on_fork, migrate_instance, prepare_cancellation_on,
    WorkflowAdvanceInput, WorkflowApprovalIntent,
};
use super::fork_continuation::fork_of;
use super::instance_cancellation::{approve, cancellation_denial, cancelled};
use super::instance_migration::{perform_approved_effect, started, supersede};
use super::*;

#[test]
fn a_cancellation_ends_only_the_copy_on_its_own_branch() {
    let (application, definition, instance, proposal, required, _) =
        approval_journey("applied", 89_000);
    let fork = fork_of(&application, instance.branch());
    assert_eq!(
        cancellation_denial(
            prepare_cancellation_on(&application, fork, instance.clone(), 89_010)
                .map(|execute| execute()),
        ),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceAffinityMismatch,
        "an instance is cancelled only on its own branch",
    );

    let successor = started(continue_on_fork(
        &application,
        fork,
        instance.clone(),
        definition,
        "propose",
        89_011,
    ))
    .instance()
    .clone();
    let ended = cancelled(cancel_instance(&application, successor.clone(), 89_012));
    assert_eq!(ended.instance(), &successor);
    assert!(ended.performed_node_paths().is_empty());

    // The copy on the source branch still takes its approval, and the same
    // key then cancels it afresh: a cancellation names its own instance.
    approve(&application, &instance, &required, &proposal, 89_013);
    let own = cancelled(cancel_instance(&application, instance.clone(), 89_012));
    assert!(!own.replayed());
    assert_eq!(own.instance(), &instance);
}

#[test]
fn a_migrated_instance_reports_the_effect_its_source_performed() {
    let (application, definition, instance, proposal, required, _) =
        approval_journey("applied", 89_100);
    approve(&application, &instance, &required, &proposal, 89_110);
    let target = supersede(&application, definition, "done", 89_111);
    perform_approved_effect(&application, &instance, 89_112);
    let successor = started(migrate_instance(
        &application,
        instance,
        target,
        "done",
        89_114,
    ))
    .instance()
    .clone();

    let ended = cancelled(cancel_instance(&application, successor.clone(), 89_115));
    assert!(!ended.replayed());
    assert_eq!(ended.performed_node_paths(), ["apply".to_owned()]);
    let replay = cancelled(cancel_instance(&application, successor.clone(), 89_115));
    assert!(replay.replayed());
    assert_eq!(
        replay.performed_node_paths(),
        ["apply".to_owned()],
        "a replay reads the carried effect back from settled history",
    );
    assert_eq!(read_dimension(application.runtime(), successor.branch()), 8);
}

#[test]
fn a_cancellation_without_the_start_capability_is_refused_and_claims_no_key() {
    let (application, _, instance, _, _, _) = approval_journey("applied", 89_200);
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let refused = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowApprovalIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&89_210_u64)
        .prepare_workflow_instance_cancellation(&application, instance.clone())
        .err();
    match refused {
        Some(WorthQueryWorkflowInstanceStartPreparationDenial::InstancePreparation(
            WorkflowInstancePreparationDenial::Attempt(attempt),
        )) => assert_eq!(
            attempt.kind(),
            WorthQueryApplicationAttemptDenialKind::WorkflowInstanceAuthorityMismatch
        ),
        other => panic!("only the start capability cancels: {other:?}"),
    }
    let ended = cancelled(cancel_instance(&application, instance, 89_210));
    assert!(!ended.replayed(), "the refused request claimed no key");
}
