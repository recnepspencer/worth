use std::sync::Arc;

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provider_session_protocol::cleanup;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provider_session_protocol::session_run;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provider_session_protocol::SessionCallCounts;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provider_session_protocol::SessionCallObservation;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provider_session_protocol::SessionFailurePoint;

#[test]
fn dropping_a_readmitted_session_aborts_once() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let calls = Arc::new(SessionCallCounts::default());
        let (mut running, graph) = session_run(
            SessionFailurePoint::None,
            Arc::clone(&calls),
            false,
            resource_request,
        );
        let installed_running_phase = execution.phase_for_direct_for_test(&running);

        drop(
            running
                .admit_provider_execution_plan(&graph)
                .expect("provider plan should admit")
                .readmit(&installed_running_phase)
                .expect("provider plan should readmit"),
        );
        assert_eq!(
            calls.observe(),
            SessionCallObservation {
                readmissions: 1,
                aborts: 1,
                ..SessionCallObservation::default()
            }
        );
        cleanup(running);
    });
}

#[test]
fn dropping_a_prepared_session_aborts_once() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let calls = Arc::new(SessionCallCounts::default());
        let (mut running, graph) = session_run(
            SessionFailurePoint::None,
            Arc::clone(&calls),
            false,
            resource_request,
        );
        let installed_running_phase = execution.phase_for_direct_for_test(&running);

        drop(
            running
                .admit_provider_execution_plan(&graph)
                .expect("provider plan should admit")
                .readmit(&installed_running_phase)
                .expect("provider plan should readmit")
                .prepare()
                .expect("provider session should prepare"),
        );
        assert_eq!(
            calls.observe(),
            SessionCallObservation {
                readmissions: 1,
                preparations: 1,
                aborts: 1,
                ..SessionCallObservation::default()
            }
        );
        cleanup(running);
    });
}

#[test]
fn dropping_a_staged_session_aborts_once() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let calls = Arc::new(SessionCallCounts::default());
        let (mut running, graph) = session_run(
            SessionFailurePoint::None,
            Arc::clone(&calls),
            false,
            resource_request,
        );
        let installed_running_phase = execution.phase_for_direct_for_test(&running);

        drop(
            running
                .admit_provider_execution_plan(&graph)
                .expect("provider plan should admit")
                .readmit(&installed_running_phase)
                .expect("provider plan should readmit")
                .prepare()
                .expect("provider session should prepare")
                .bind_reads_and_effects(),
        );
        assert_eq!(
            calls.observe(),
            SessionCallObservation {
                readmissions: 1,
                preparations: 1,
                aborts: 1,
                ..SessionCallObservation::default()
            }
        );
        cleanup(running);
    });
}

#[test]
fn dropping_a_commit_prepared_session_aborts_once() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let calls = Arc::new(SessionCallCounts::default());
        let (mut running, graph) = session_run(
            SessionFailurePoint::None,
            Arc::clone(&calls),
            false,
            resource_request,
        );
        let installed_running_phase = execution.phase_for_direct_for_test(&running);

        drop(
            running
                .admit_provider_execution_plan(&graph)
                .expect("provider plan should admit")
                .readmit(&installed_running_phase)
                .expect("provider plan should readmit")
                .prepare()
                .expect("provider session should prepare")
                .bind_reads_and_effects()
                .prepare_for_commit_for_test()
                .expect("staged session should prepare for commit"),
        );
        assert_eq!(
            calls.observe(),
            SessionCallObservation {
                readmissions: 1,
                preparations: 1,
                staged_preparations: 1,
                aborts: 1,
                ..SessionCallObservation::default()
            }
        );
        cleanup(running);
    });
}

#[test]
fn failed_preparation_aborts_once() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let calls = Arc::new(SessionCallCounts::default());
        let (mut running, graph) = session_run(
            SessionFailurePoint::PreparationPanic,
            Arc::clone(&calls),
            false,
            resource_request,
        );
        let installed_running_phase = execution.phase_for_direct_for_test(&running);

        let _ = running
            .admit_provider_execution_plan(&graph)
            .expect("provider plan should admit")
            .readmit(&installed_running_phase)
            .expect("provider plan should readmit")
            .prepare()
            .expect_err("preparation panic must be contained");
        assert_eq!(
            calls.observe(),
            SessionCallObservation {
                readmissions: 1,
                preparations: 1,
                aborts: 1,
                ..SessionCallObservation::default()
            }
        );
        cleanup(running);
    });
}
