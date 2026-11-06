use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::unwind_custody::assert_idempotency_refuses_unpublished;

use std::num::NonZeroUsize;
use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
#[test]
fn world_unwind_retains_and_releases_the_exact_query_idempotency_slot() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&phase.test_host_request_for_test(), &world, &request)
        })
        .expect("installed host call opens");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &phase.test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
        })
        .expect("installed host call opens");
    let program = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "post-relational-unwind",
            )
        })
        .expect("installed host call opens");
    let retry_while_retained = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "post-relational-unwind",
            )
        })
        .expect("installed host call opens");
    let retry_after_cleanup = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "post-relational-unwind",
            )
        })
        .expect("installed host call opens");
    let before_inspections =
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::unpublished_idempotency::reservation::unwind_recovery_inspection_count();
    world
        .application
        .product_runtime()
        .operation_control_for_test()
        .panic_before_product_compare_once_for_test();

    let unwind = catch_unwind(AssertUnwindSafe(|| {
        world.application.compare_and_commit_application(
            program,
            idempotency(0xD1, 0xD1),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    }));
    let isolated = unwind.expect("the application boundary isolates provider unwind");
    assert!(
        matches!(
            isolated,
            WorthQueryApplicationCommitOutcome::Indeterminate(_)
        ),
        "provider unwind is typed without claiming a product commit: {isolated:?}",
    );
    assert_eq!(
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::unpublished_idempotency::reservation::unwind_recovery_inspection_count(),
        before_inspections + 1,
        "only the unwind guard inspects its exact World recovery identity",
    );
    assert_eq!(
        (*(*world.application.runtime_for_test()).primary_provider_for_test())
            .unpublished_idempotency_count(),
        1,
    );

    let page = world
        .application
        .product_publication_recovery_page(None, NonZeroUsize::new(1).unwrap())
        .unwrap();
    let [row] = page.rows() else {
        panic!("World must retain the exact owner movement abandoned by unwind")
    };
    let recovery = world
        .application
        .readmit_product_publication_recovery(row.handle())
        .unwrap();
    let before_retry_inspections =
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::unpublished_idempotency::reservation::unwind_recovery_inspection_count();
    assert_idempotency_refuses_unpublished(world.application.compare_and_commit_application(
        retry_while_retained,
        idempotency(0xD1, 0xD1),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    ));
    assert_eq!(
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::unpublished_idempotency::reservation::unwind_recovery_inspection_count(),
        before_retry_inspections,
        "ordinary idempotency resolution must not inspect World recovery",
    );
    let receipt = world
        .application
        .release_product_publication_recovery(recovery, 0)
        .unwrap();
    assert_eq!(receipt.retired_component_count(), 0);
    assert_eq!(
        (*(*world.application.runtime_for_test()).primary_provider_for_test())
            .unpublished_idempotency_count(),
        0,
        "cleanup releases Query's exact recovery-indexed slot",
    );

    let after = world.application.compare_and_commit_application(
        retry_after_cleanup,
        idempotency(0xD1, 0xD1),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::Denied(denial) = after else {
        panic!("the unpublished Relational movement leaves the product basis stale: {after:?}")
    };
    assert_ne!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::Idempotency,
        "cleanup removes the retained idempotency denial before fresh admission",
    );
}
