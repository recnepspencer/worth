//! Canonical Query terminal carriage from World's performed publication.

use super::application_attempt::{authenticated_principal, idempotency, resolved_account};
use super::fixture::{installed_authorization_world, live_scope};
use crate::domain_computation::primary_graph::{
    recoverable_application_world, two_recoverable_application_commits,
    WorthQueryApplicationCommitOutcome,
};

#[test]
fn receipt_recovery_and_dispatch_observation_share_the_exact_world_terminal() {
    let (world, receipt) = recoverable_application_world(191, "terminal-carriage");
    let committed = receipt.committed_product_publication();
    assert_eq!(
        committed.product_branch().owner_identity(),
        committed.composite_commit().owner_identity()
    );
    assert_eq!(
        committed.product_branch().owner_identity(),
        committed.publication_attempt().owner_identity()
    );

    let observation = world
        .application
        .observe_committed_dispatch_outbox(&receipt)
        .unwrap()
        .expect("the committed operation carries its outbox");
    assert_eq!(observation.committed_product_publication(), committed);

    let handle = world
        .application
        .mint_recovery_handle(&receipt)
        .expect("the committed aftermath admits recovery");
    assert_eq!(handle.binding().committed_product_publication(), committed);
}

#[test]
fn same_world_publications_retain_distinct_commit_and_attempt_occurrences() {
    let (world, first, second) = two_recoverable_application_commits(192, 193);
    let first_world = first.committed_product_publication();
    let second_world = second.committed_product_publication();
    assert_eq!(first_world.product_branch(), second_world.product_branch());
    assert_eq!(
        first_world.product_incarnation(),
        second_world.product_incarnation()
    );
    assert_ne!(
        first_world.composite_commit(),
        second_world.composite_commit()
    );
    assert_ne!(
        first_world.publication_attempt(),
        second_world.publication_attempt()
    );

    let first_observed = world
        .application
        .observe_committed_dispatch_outbox(&first)
        .unwrap()
        .unwrap();
    let second_observed = world
        .application
        .observe_committed_dispatch_outbox(&second)
        .unwrap()
        .unwrap();
    assert_eq!(first_observed.committed_product_publication(), first_world);
    assert_eq!(
        second_observed.committed_product_publication(),
        second_world
    );
}

#[test]
fn idempotent_recovery_reuses_the_original_world_terminal() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let program = || {
        super::application_attempt::preimage_evidence::retained_status_program(
            &world,
            &principal,
            &account,
            &request,
            "recovered-world-terminal",
            super::application_attempt::preimage_evidence::RetentionMutationBreadth::Narrow,
        )
    };
    let first = program();
    let retry = program();
    let binding = idempotency(194, 195);
    let WorthQueryApplicationCommitOutcome::Committed(committed) = world
        .application
        .compare_and_commit_application(first, binding)
    else {
        panic!("first application must commit")
    };
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) = world
        .application
        .compare_and_commit_application(retry, binding)
    else {
        panic!("retry must recover the committed terminal")
    };

    assert_eq!(
        recovered.committed_product_publication(),
        committed.committed_product_publication()
    );
    assert_eq!(recovered.dispatch_outbox(), committed.dispatch_outbox());
    assert!(recovered.is_same_authoritative_commit(&committed));
}

#[test]
fn equal_relational_receipt_axes_from_different_worlds_are_not_one_commit() {
    let (_first_world, first) = recoverable_application_world(196, "equal-relational-axes");
    let (_second_world, second) = recoverable_application_world(196, "equal-relational-axes");
    assert_eq!(first.commit_reference(), second.commit_reference());
    let second =
        second.with_provider_runtime_instance_id_for_test(first.provider_runtime_instance_id());
    assert_ne!(
        first.committed_product_publication(),
        second.committed_product_publication()
    );
    assert!(!first.is_same_authoritative_commit(&second));
}
