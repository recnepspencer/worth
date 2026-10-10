//! idempotency without the claimed causal fact is not equivalent.
use super::*;

#[test]
fn idempotency_without_the_claimed_causal_fact_is_not_equivalent() {
    use crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality;

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
    let first = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "plain-commit",
            )
        })
        .expect("the fixture read owns its installed host call");
    let retry = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "plain-commit",
            )
        })
        .expect("the fixture read owns its installed host call");
    let parent = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .product()
                .relational_basis()
                .observation()
                .commit_receipt()
                .cloned()
                .expect("fixture head")
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.compare_and_commit_application(
            first,
            idempotency(30, 30),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));

    let WorthQueryApplicationCommitOutcome::Denied(denial) = world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            world
                .application
                .compare_and_commit_application_with_aftermath(
                    phase,
                    retry,
                    idempotency(30, 30),
                    WorthQueryPendingAftermathCausality::undo_of(parent),
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
        })
        .expect("fixture owner admits its advancement")
    else {
        panic!("a plain idempotency row cannot impersonate a causal commit");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
    );
}
