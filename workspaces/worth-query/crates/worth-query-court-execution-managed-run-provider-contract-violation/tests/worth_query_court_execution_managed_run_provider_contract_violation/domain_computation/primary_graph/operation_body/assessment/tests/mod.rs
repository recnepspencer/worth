//! The returned result, rather than the producing stack frame, owns its charge.
use worth_query_execution::facade::domain_computation::WorthQueryCurrentBodyRequirementAssessment;
use worth_query_execution::facade::domain_computation::WorthQueryOperationBodyRequirementFinding;
use crate::domain_computation::primary_graph::operation_body::assessment::tests;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::QueryRequestExecution;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;

pub fn returned_findings(
    execution: &QueryRequestExecution<'_>,
) -> WorthQueryCurrentBodyRequirementAssessment {
    let memory = execution
        .reserve(1024)
        .expect("bounded finding reservation");
    WorthQueryCurrentBodyRequirementAssessment::new_for_test(
        vec![WorthQueryOperationBodyRequirementFinding::new(
            ApplicationFieldBindingLocus::new("Account", "AccountPolicy", "AccountNote"),
            false,
        )],
        memory,
    )
}

#[test]
fn returned_findings_retain_native_request_memory_until_the_result_drops() {
    let world = installed_authorization_world(true);
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    worth_query_execution::facade::domain_computation::place_managed_computations_on_this_thread_for_test(
        worth_query_execution::facade::domain_computation::WorthQueryExecutionPlacementForTest::Serial,
    );
    worth_query_execution::facade::domain_computation::bound_advancement_requests_on_this_thread_for_test(Some(
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            Some(64 * 1024),
            None,
        ),
    ));
    world
        .application
        .with_host_advancement(|phase| {
            let execution = phase
                .execution_for_for_test(&(*world.application.product_runtime_for_test()))
                .unwrap();
            let available = match execution.reserve(1_u64 << 60) {
                Err(Resource::MemoryLimit { admitted, .. }) => admitted,
                Err(cause) => panic!("expected policy quote: {cause:?}"),
                Ok(_) => panic!("oversized quote must refuse without allocating"),
            };
            assert!(available > 1024);
            let result = returned_findings(execution);
            assert_eq!(result.findings().len(), 1);
            match execution.reserve(available) {
                Err(Resource::MemoryLimit { admitted, .. }) => {
                    assert_eq!(admitted + 1024, available)
                }
                Err(cause) => panic!("unexpected reservation cause: {cause:?}"),
                Ok(_) => panic!("returned findings released their native claim too early"),
            }
            drop(result);
            assert!(
                execution.reserve(available).is_ok(),
                "dropping findings releases exactly their charge"
            );
        })
        .unwrap();
}
