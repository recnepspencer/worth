use super::{
    admitted_program, authenticated_principal, idempotency, installed_authorization_world,
    live_scope, resolved_account,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialStage, WorthQueryApplicationCommitOutcome,
    WorthQueryProducerInvariantRequirement,
};
use worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint;

const REQUIRED: &[WorthQueryProducerInvariantRequirement] =
    &[WorthQueryProducerInvariantRequirement::new(
        "uninstalled-producer-rule",
        1,
        0,
        ApplicationInvariantExecutionPoint::CommitBoundary,
    )];

#[test]
fn producer_requirement_denies_before_first_publication_and_leaves_retry_clean() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let candidate = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(&phase, &world, &principal, &account, &request, "published")
                .with_producer_required_invariants(REQUIRED)
        })
        .expect("the fixture read owns its installed host call");
    let outcome = world.application.compare_and_commit_application(
        candidate,
        idempotency(231, 231),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("missing producer rule published a candidate: {outcome:?}");
    };
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    let _predecessor = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &live_scope())
        })
        .expect("the fixture read owns its installed host call");

    let retry = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(&phase, &world, &principal, &account, &request, "published")
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.compare_and_commit_application(
            retry,
            idempotency(231, 231),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
}
