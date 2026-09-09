use super::{
    authenticated, idempotency, link_program, live_scope, resolved_account, resolved_principal,
    WorthQueryApplicationCommitOutcome,
};
use crate::domain_computation::primary_graph::tests::fixture::installed_two_principal_authorization_world;

#[test]
fn unrelated_adjacency_growth_requires_fresh_product_admission_for_unchanged_facts() {
    let world = installed_two_principal_authorization_world(false);
    let request = live_scope();
    let alice = authenticated(&world, "alice", &request);
    let bob = authenticated(&world, "bob", &request);
    let alice_identity = resolved_principal(&world, 1, &request);
    let bob_identity = resolved_principal(&world, 2, &request);
    let first_account = resolved_account(&world, "open", &request);
    let second_account = resolved_account(&world, "unrelated", &request);
    let selected = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let commit_count = || {
        world
            .application
            .primary_provider
            .graph
            .with_runtime(|runtime| runtime.history().immutable_commit_count())
    };
    let baseline = commit_count();
    let alice_program = link_program(
        &world,
        &alice,
        &alice_identity,
        &first_account,
        &request,
        "open",
        "alice-owner",
    );
    let bob_program = link_program(
        &world,
        &bob,
        &bob_identity,
        &second_account,
        &request,
        "unrelated",
        "bob-owner",
    );
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(bob_program, idempotency(33, 33)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let current = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    assert_ne!(current.selected_commit(), selected.selected_commit());
    assert_eq!(commit_count(), baseline + 1);

    let outcome = world
        .application
        .compare_and_commit_application(alice_program, idempotency(34, 34));
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("unrelated adjacency growth changes product currentness only: {outcome:?}");
    };
    assert_eq!(
        denial.kind(),
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind::ProductBasisStale
    );
    assert_eq!(
        denial.stage(),
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    assert_eq!(commit_count(), baseline + 1);
    assert_eq!(
        world
            .application
            .product_runtime()
            .admit_product_branch(world.application.product_runtime().default_branch())
            .unwrap()
            .selected_commit(),
        current.selected_commit()
    );

    let readmitted = link_program(
        &world,
        &alice,
        &alice_identity,
        &first_account,
        &request,
        "open",
        "alice-owner",
    );
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(readmitted, idempotency(34, 34)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert_eq!(commit_count(), baseline + 2);
}
