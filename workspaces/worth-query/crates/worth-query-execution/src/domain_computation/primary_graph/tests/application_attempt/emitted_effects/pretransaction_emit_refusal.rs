use super::*;

#[test]
fn rejection_before_transaction_publishes_no_emit_causality() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let program = admitted_program_with_emit(
        &world,
        &principal,
        &account,
        &request,
        "rejected",
        Some("must-not-publish"),
    );

    world.faults.reject_next_commit_before_transaction();
    let outcome = world.application.compare_and_commit_application(
        program,
        idempotency(32, 32),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        !matches!(
            outcome,
            WorthQueryApplicationCommitOutcome::Committed(_)
                | WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
        ),
        "rejected transaction claimed commit: {outcome:?}"
    );
    assert_eq!(
        world
            .application
            .primary_provider
            .published_application_commit_count(),
        0
    );
}
