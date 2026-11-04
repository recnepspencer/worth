//! Performed receipts remain available independently of a synthetic byte window.

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;

#[test]
fn held_completed_evidence_preserves_exact_old_replay_without_eviction() {
    {
        let world = installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
        let request = live_scope();

        let principal = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                authenticated_principal(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
                    &world,
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");
        let account = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                resolved_account(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
                    &world,
                    "open",
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");
        let original = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                admitted_program(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &world,
                    &principal,
                    &account,
                    &request,
                    "h91",
                )
            })
            .expect("the fixture read owns its installed host call");
        let replay = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                admitted_program(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &world,
                    &principal,
                    &account,
                    &request,
                    "h91",
                )
            })
            .expect("the fixture read owns its installed host call");
        let first_outcome = world.application.compare_and_commit_application(
            original,
            idempotency(91, 91),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        );
        let WorthQueryApplicationCommitOutcome::Committed(first) = first_outcome else {
            panic!("the original prepared intent must commit: {first_outcome:?}");
        };
        let mut held = Vec::new();
        held.push(first);
        let mut state = "h91".to_owned();
        for key in (93..).step_by(2).take(63) {
            let account = world
                .application
                .with_host_advancement(|phase| {
                    let host_custody = phase.test_host_request_for_test();
                    resolved_account(
                        &(host_custody.phase_for_application_for_test(&world.application))
                            .test_host_request_for_test(),
                        &world,
                        &state,
                        &request,
                    )
                })
                .expect("the fixture read owns its installed host call");
            let next = format!("h{key}");
            let program = world
                .application
                .with_host_advancement(|phase| {
                    let host_custody = phase.test_host_request_for_test();
                    admitted_program(
                        &host_custody.phase_for_application_for_test(&world.application),
                        &world,
                        &principal,
                        &account,
                        &request,
                        &next,
                    )
                })
                .expect("the fixture read owns its installed host call");
            let WorthQueryApplicationCommitOutcome::Committed(receipt) =
                world.application.compare_and_commit_application(
                    program,
                    idempotency(key, key),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
            else {
                panic!("each actual status edit commits while prior receipts remain held")
            };
            held.push(receipt);
            state = next;
        }
        let first = &held[0];
        let original_work = first.mutation_work().expect("commit-derived work");
        assert!(!original_work.touched_records().is_empty());
        let head = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                world
                    .selected_product(
                        &(host_custody.phase_for_application_for_test(&world.application))
                            .test_host_request_for_test(),
                    )
                    .product()
                    .selected_commit()
                    .clone()
            })
            .expect("the fixture read owns its installed host call");
        let outcome = world.application.compare_and_commit_application(
            replay,
            idempotency(91, 91),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        );
        let WorthQueryApplicationCommitOutcome::AlreadyCommitted(replayed) = outcome else {
            panic!("the original prepared retry must recover its retained receipt: {outcome:?}")
        };
        assert_eq!(held.len(), 64);
        assert!(replayed.is_same_authoritative_commit(first));
        assert_eq!(replayed.commit_reference(), first.commit_reference());
        assert_eq!(replayed.mutation_work(), Some(original_work));
        world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                assert_eq!(
                    world
                        .selected_product(
                            &(host_custody.phase_for_application_for_test(&world.application))
                                .test_host_request_for_test(),
                        )
                        .product()
                        .selected_commit(),
                    &head
                )
            })
            .expect("the fixture read owns its installed host call");
    }
}
