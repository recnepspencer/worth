//! Canonical Query terminal carriage from World's performed publication.

use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::application_attempt::compare_and_commit::commit_receipt::certification::CertificationWorthQueryApplicationCommitReceiptForTest;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::recoverable_application_world;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::two_recoverable_application_commits;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;

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
        .with_host_advancement(|phase| {
            world
                .application
                .observe_committed_dispatch_outbox(&phase, &receipt)
                .unwrap()
                .expect("the committed operation carries its outbox")
        })
        .expect("the fixture read owns its installed host call");
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
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();

            let first_observed = world
                .application
                .observe_committed_dispatch_outbox(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &first,
                )
                .unwrap()
                .unwrap();
            let second_observed = world
                .application
                .observe_committed_dispatch_outbox(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &second,
                )
                .unwrap()
                .unwrap();
            assert_eq!(first_observed.committed_product_publication(), first_world);
            assert_eq!(
                second_observed.committed_product_publication(),
                second_world
            );
        })
        .expect("the fixture block owns its installed host call")
}

#[test]
fn idempotent_recovery_reuses_the_original_world_terminal() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let program = || {
        world
            .application
            .with_host_advancement(|phase| {
                worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::retained_status_program(
                    &(phase).test_host_request_for_test(),
                    &world,
                    &principal,
                    &account,
                    &request,
                    "recovered-world-terminal",
                    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::RetentionMutationBreadth::Narrow,
                )
            })
            .expect("the fixture invocation owns its installed host call")
    };
    let first = program();
    let retry = program();
    let binding = idempotency(194, 195);
    let WorthQueryApplicationCommitOutcome::Committed(committed) =
        world.application.compare_and_commit_application(
            first,
            binding,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("first application must commit")
    };
    let WorthQueryApplicationCommitOutcome::RecoveredCommitted(recovered) =
        world.application.compare_and_commit_application(
            retry,
            binding,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("retry must recover the committed terminal")
    };

    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::recorded_receipt::assert_original(
        &recovered, &committed,
    );
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
