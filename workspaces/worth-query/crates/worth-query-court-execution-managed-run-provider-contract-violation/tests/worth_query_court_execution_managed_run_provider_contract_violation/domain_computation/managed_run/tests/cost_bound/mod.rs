use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::cost_bound::UNRELATED_RUN_COUNT;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::admitted_plan;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::cost_bound::assert_exact_admission_work;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::cost_bound::assert_exact_direct_readmission_work;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::cost_bound::assert_exact_preflight_denial_work;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::cost_bound::assert_exact_workflow_readmission_work;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::cost_bound::assert_rejection_work_is_constant_with_unrelated_authority;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::cost_bound::denied_readmission_work;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::operation_binding::authority::tests::direct_authority;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::cost_bound::direct_readmission_work;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::query_runtime;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::cost_bound::unrelated_artifact_run_in_runtime;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::cost_bound::workflow_readmission_work;

#[test]
fn managed_run_work_is_invariant_to_unrelated_live_authority_width() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let disposed = Arc::new(AtomicUsize::new(0));
    let runtime = query_runtime();
    let unrelated = (0..UNRELATED_RUN_COUNT)
        .map(|index| {
            unrelated_artifact_run_in_runtime(
                &runtime,
                index,
                Arc::clone(&disposed),
                resource_request,
            )
        })
        .collect::<Vec<_>>();

    let plan = admitted_plan("cost-bound-target", 8);
    let operation = direct_authority(&runtime, &plan);
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("target operation should start");
    let lower = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::managed_run::tests::causal_fixture::managed_admission_context(&resource_request);
    let admitted = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&operation, attempt, lower.read_request(), resource_request)
        .expect("target run should admit independently of unrelated authority width");

    assert_exact_admission_work(admitted.counters());
    let terminal = admitted
        .start()
        .completed()
        .expect("target run with no provider work should complete");
    assert_exact_admission_work(terminal.counters());
    assert_eq!(terminal.provider_work().issued_call_count(), 0);
    let cleanup = terminal.cleanup().expect("target cleanup should complete");
    assert_exact_admission_work(cleanup.inspection().counters());
    assert_eq!(cleanup.inspection().released_reservation_count(), 1);
    assert!(cleanup.inspection().resources_released());
    assert_eq!(disposed.load(Ordering::Acquire), 0);

    assert_rejection_work_is_constant_with_unrelated_authority(resource_request);
    assert_eq!(disposed.load(Ordering::Acquire), 0);

    drop(unrelated);
    assert_eq!(disposed.load(Ordering::Acquire), UNRELATED_RUN_COUNT);
}
#[test]
fn readmission_work_is_invariant_to_same_runtime_unrelated_authority_width() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let direct_empty = direct_readmission_work(execution, 0, resource_request);
        let direct_wide = direct_readmission_work(execution, UNRELATED_RUN_COUNT, resource_request);
        assert_eq!(direct_empty, direct_wide);
        assert_exact_direct_readmission_work(direct_wide);

        let workflow_empty = workflow_readmission_work(execution, 0, resource_request);
        let workflow_wide =
            workflow_readmission_work(execution, UNRELATED_RUN_COUNT, resource_request);
        assert_eq!(workflow_empty, workflow_wide);
        assert_exact_workflow_readmission_work(workflow_wide);

        let denial_empty = denied_readmission_work(execution, 0, resource_request);
        let denial_wide = denied_readmission_work(execution, UNRELATED_RUN_COUNT, resource_request);
        assert_eq!(denial_empty, denial_wide);
        assert_exact_preflight_denial_work(denial_wide);
    });
}
