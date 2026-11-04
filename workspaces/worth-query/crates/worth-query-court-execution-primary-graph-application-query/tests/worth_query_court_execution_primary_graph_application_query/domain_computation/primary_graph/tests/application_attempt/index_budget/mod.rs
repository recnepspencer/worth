use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;

#[test]
fn pre_effect_index_budget_denial_leaves_no_effect_and_retries_through_query() {
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
    let denied = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "budget-replacement",
            )
        })
        .expect("the fixture read owns its installed host call");

    // Constrain only the next ordinary preflight. The real Relational candidate
    // and index preparation still run; no World publication is faulted.
    world.faults.constrain_next_index_maintenance_budget();
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            denied,
            idempotency(41, 41),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("index work exhaustion must deny before a World effect");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::IndexMaintenanceBudgetExceeded
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::ProviderCommit
    );
    assert_eq!(
        (*(*world
            .application
            .runtime_for_test())
            .primary_provider_for_test())
            .published_application_commit_count_for_test(),
        0
    );
    let _still_open = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &live_scope(),
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
                "budget-replacement",
            )
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryApplicationCommitOutcome::Committed(receipt) =
        world.application.compare_and_commit_application(
            retry,
            idempotency(41, 41),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the identical intent must commit when ordinary index work is admitted");
    };
    let index_work = receipt
        .mutation_work()
        .expect("the committed receipt retains mutation work")
        .index_maintenance_work();
    assert!(index_work.work_units > 0);
    assert_eq!(index_work.cold_record_slots, 0);
    assert!(index_work.generation_publications_reserved > 0);
    let _committed = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "budget-replacement",
                &live_scope(),
            )
        })
        .expect("the fixture read owns its installed host call");
}
