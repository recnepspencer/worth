pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;

pub use std::time::Duration;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::program_roster::installed_program_support;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::program_required_binding::ProgramRequiredInput;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ProgramRequiredOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitTerminalKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
pub(crate) use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_operation;
pub(crate) use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_mutation_free_program;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program_on_selected;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program_with_emit;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program_with_expected_status;
pub(crate) use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::decision_currentness::assert_changed_decision;
pub(crate) use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::decision_currentness::assert_product_basis_stale;
#[test]
fn concurrent_equivalent_attempts_publish_one_transaction() {
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
            admitted_program(&phase, &world, &principal, &account, &request, "once")
        })
        .expect("the fixture read owns its installed host call");
    let second = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(&phase, &world, &principal, &account, &request, "once")
        })
        .expect("the fixture read owns its installed host call");

    let (left, right) = std::thread::scope(|scope| {
        let left = scope.spawn(|| {
            world.application.compare_and_commit_application(
                first,
                idempotency(12, 12),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
        });
        let right = scope.spawn(|| {
            world.application.compare_and_commit_application(
                second,
                idempotency(12, 12),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
        });
        (left.join().unwrap(), right.join().unwrap())
    });
    let receipts = [left, right]
        .into_iter()
        .map(|outcome| match outcome {
            WorthQueryApplicationCommitOutcome::Committed(receipt)
            | WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) => receipt,
            unexpected => panic!("unexpected concurrent outcome: {unexpected:?}"),
        })
        .collect::<Vec<_>>();
    assert!(receipts[0].is_same_authoritative_commit(&receipts[1]));
    let terminal_kinds = receipts
        .iter()
        .map(|receipt| receipt.terminal().kind())
        .collect::<Vec<_>>();
    assert!(terminal_kinds.contains(&WorthQueryApplicationCommitTerminalKind::Executed));
    assert!(terminal_kinds.contains(&WorthQueryApplicationCommitTerminalKind::Recovered));
}
#[test]
fn preparation_commit_recovery_and_retry_perform_no_execution_digest_derivation() {
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
    let before = worth_query_execution::court_doors_for_test::execution_digest::test_hash_parts_call_count();
    let first = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "digest-free",
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
                "digest-free",
            )
        })
        .expect("the fixture read owns its installed host call");

    world.faults.lose_next_commit_response();
    assert!(matches!(
        world.application.compare_and_commit_application(
            first,
            idempotency(31, 31),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let after_commit = worth_query_execution::court_doors_for_test::execution_digest::test_hash_parts_call_count();
    assert_eq!(
        after_commit,
        before,
        "preparation, provider execution, commit, or response-loss recovery derived legacy execution digests: {:?}",
        worth_query_execution::court_doors_for_test::execution_digest::test_hash_parts_domains_since(before)
    );

    assert!(matches!(
        world.application.compare_and_commit_application(
            retry,
            idempotency(31, 31),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
    ));
    assert_eq!(
        worth_query_execution::court_doors_for_test::execution_digest::test_hash_parts_call_count(),
        after_commit,
        "idempotent retry or recovery derived a legacy execution digest"
    );
}
