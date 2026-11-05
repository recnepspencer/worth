use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provider_session_protocol::GraphOnlyProvider;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provider_session_protocol::SessionCallCounts;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provider_session_protocol::SessionFailurePoint;
use worth_query_installation::facade::WorthQueryOperationGraphAccess;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provider_session_protocol::cleanup;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provider_session_protocol::closed_session_affinity;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::managed_graph_run_with_provider;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provider_session_protocol::session_run;

use crate::domain_computation::managed_run::tests::*;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use worth_query_execution::court_doors_for_test::domain_computation::provider_session::protocol::session_affinity::WorthQueryProviderSessionAffinityIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryProviderExecutionPlanView;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionFailure;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionLifecycle;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionProtocolCounters;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionProtocolStage;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionRecoveryPosture;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionToken;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionTokenAdmission;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionView;
use worth_query_execution::facade::domain_computation::WorthQuerySessionAbortOutcome;
#[test]
fn sealed_plan_prepares_session_binds_work_and_aborts() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let calls = Arc::new(SessionCallCounts::default());
        let (mut running, graph) = session_run(
            SessionFailurePoint::None,
            Arc::clone(&calls),
            true,
            resource_request,
        );
        let installed_running_phase = execution.phase_for_direct_for_test(&running);

        let expected_run = running.identity().to_owned();
        let expected_basis = running
            .provider_plan_bridge_basis_for_test()
            .identity()
            .as_str()
            .to_owned();
        let expected_snapshot = running.execution_snapshot_reference_for_test();
        {
            let plan = running
                .admit_provider_execution_plan(&graph)
                .expect("exact managed authorities should admit a provider plan");
            assert_eq!(plan.contract().provider_role(), "managed-graph");
            assert_eq!(plan.contract().managed_run_identity(), expected_run);
            assert_eq!(plan.contract().execution_basis_identity(), expected_basis);
            assert_eq!(plan.contract().snapshot_identity(), expected_snapshot);
            assert_eq!(
                plan.contract().graph_authority_identity(),
                graph.authority_identity()
            );
            assert_eq!(plan.contract().effect_closure(), ["mutation"]);
            assert_eq!(plan.counters().authority_checks(), 1);
            let readmitted = plan
                .readmit(&installed_running_phase)
                .expect("provider should readmit the plan");
            let prepared = readmitted
                .prepare()
                .expect("provider should prepare the session");
            let staged = prepared.bind_reads_and_effects();
            assert_eq!(
                staged.read_authority().token_identity(),
                staged.effect_authority().token_identity()
            );
            assert_eq!(staged.counters().provider_calls(), 2);
            let outcome = staged.abort();
            assert_eq!(
                outcome.recovery_posture(),
                WorthQueryProviderSessionRecoveryPosture::Closed
            );
            assert!(matches!(outcome, WorthQuerySessionAbortOutcome::Aborted(_)));
        }
        assert_eq!(calls.readmissions.load(Ordering::Acquire), 1);
        assert_eq!(calls.preparations.load(Ordering::Acquire), 1);
        assert_eq!(calls.aborts.load(Ordering::Acquire), 1);
        cleanup(running);
    });
}
#[test]
fn independently_admitted_sessions_carry_distinct_opaque_affinities() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        assert_ne!(
            closed_session_affinity(execution, resource_request),
            closed_session_affinity(execution, resource_request)
        );
    });
}
#[test]
fn provider_without_session_lifecycle_is_denied_before_any_protocol_call() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (mut running, graph) = managed_graph_run_with_provider(
        WorthQueryOperationGraphAccess::Observe,
        GraphOnlyProvider,
        resource_request,
    );
    let failure = running
        .admit_provider_execution_plan(&graph)
        .expect_err("one-shot-only provider must not enter the session lane");
    assert_eq!(
        failure.kind(),
        WorthQueryProviderSessionDenialKind::SessionProtocolUnsupported
    );
    cleanup(running);
}
