use worth_query_execution::facade::domain_computation::WorthQueryDirectGraphStepOutcome;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::managed_run::tests::direct_yield_cleanup_fixture::complete_direct_yield_cleanup;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::query_runtime;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::readmission_direct::yielded_direct;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::readmission_direct::yielded_direct_with_provider;

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::yield_fixture::YieldProvider;

#[test]
fn direct_readmission_mints_fresh_attempts_and_transfers_capacity() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime) = yielded_direct(execution, resource_request);
        let logical = yielded.inspection().logical_run_identity().to_owned();
        let managed_attempt = yielded.inspection().yielded_attempt_identity().to_owned();
        let resource_attempt = yielded.inspection().yielded_attempt_identity().to_owned();
        let provider_session = yielded.inspection().provider_session_identity().to_owned();
        let reservation_count = yielded.inspection().retained_capacity_reservation_count();
        let readmitted = {
            let fixture_phase_result = {
                let installed_fixture_phase = yielded.fixture_phase_for_test(active_request);
                yielded.readmit_same_runtime(&installed_fixture_phase, &runtime, &bridge)
            };
            match fixture_phase_result {
                worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionOutcome::Readmitted(
                    readmitted,
                ) => readmitted,
                _ => panic!("same-runtime readmission should succeed"),
            }
        };
        let active = readmitted.into_active();
        assert_eq!(active.logical_run_identity(), logical);
        assert_ne!(active.run_identity(), managed_attempt);
        assert_ne!(active.resource_attempt_identity(), resource_attempt);
        assert_ne!(active.provider_session_identity(), provider_session);
        assert_eq!(
            active.retained_capacity_reservation_count(),
            reservation_count
        );
        assert_eq!(reservation_count, 2);
        assert!(!active.provider_call_identity().is_empty());
        let readmitted_provider_session = active.provider_session_identity().to_owned();

        let completion = {
            let fixture_phase_result = {
                let installed_fixture_phase = active.fixture_phase_for_test(execution);
                active.advance(&installed_fixture_phase)
            };
            match fixture_phase_result {
                WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
                _ => panic!("restored provider did not complete"),
            }
        };
        let terminal = completion.into_running().completed().unwrap();
        assert_eq!(terminal.logical_run_identity(), logical);
        assert_eq!(terminal.provider_work().completed_work_units(), 4);
        assert_eq!(
            terminal.provider_work().provider_session_identity(),
            readmitted_provider_session
        );
        let cleanup = terminal.cleanup().expect("readmitted run should clean up");
        assert_eq!(
            cleanup.inspection().provider_session_identity(),
            readmitted_provider_session
        );
        assert_eq!(
            cleanup
                .inspection()
                .provider_work()
                .provider_session_identity(),
            readmitted_provider_session
        );
    });
}
#[test]
fn query_preflight_denial_returns_the_exact_yielded_capability_without_fresh_work() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime) = yielded_direct(execution, resource_request);
        let checkpoint = yielded.inspection().checkpoint().identity().to_owned();
        let resource_attempt = yielded.inspection().yielded_attempt_identity().to_owned();
        let foreign_runtime = query_runtime();
        let denied = {
            let fixture_phase_result = {
                let installed_fixture_phase = yielded.fixture_phase_for_test(active_request);
                yielded.readmit_same_runtime(&installed_fixture_phase, &foreign_runtime, &bridge)
            };
            match fixture_phase_result {
                worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionOutcome::Denied(denied) => {
                    denied
                }
                _ => panic!("foreign Query runtime should deny"),
            }
        };
        assert_eq!(
            denied.kind(),
            worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionDenialKind::ForeignQueryRuntime
        );
        let evidence = denied.readmission_evidence();
        let counters = evidence.query_counters();
        assert_eq!(counters.preflight_check_count(), 1);
        assert_eq!(counters.fresh_resource_attempt_count(), 0);
        assert_eq!(counters.bridge_readmission_attempt_count(), 0);
        assert!(evidence.bridge_counters().is_none());
        let yielded = denied.into_yielded();
        assert_eq!(yielded.inspection().checkpoint().identity(), checkpoint);
        assert_eq!(
            yielded.inspection().yielded_attempt_identity(),
            resource_attempt
        );

        let active = {
            let fixture_phase_result = {
                let installed_fixture_phase = yielded.fixture_phase_for_test(active_request);
                yielded.readmit_same_runtime(&installed_fixture_phase, &runtime, &bridge)
            };
            match fixture_phase_result {
                worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionOutcome::Readmitted(
                    readmitted,
                ) => readmitted.into_active(),
                _ => panic!("returned yielded capability should remain readmittable"),
            }
        };
        let completion = {
            let fixture_phase_result = {
                let installed_fixture_phase = active.fixture_phase_for_test(execution);
                active.advance(&installed_fixture_phase)
            };
            match fixture_phase_result {
                WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
                _ => panic!("restored provider did not complete"),
            }
        };
        assert!(completion
            .into_running()
            .completed()
            .unwrap()
            .cleanup()
            .is_ok());
    });
}
#[test]
fn provider_restore_denial_preserves_the_exact_checkpoint_and_capacity_package() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime) = yielded_direct_with_provider(
            execution,
            YieldProvider::checkpoint_restore_failure(7),
            resource_request,
        );
        let checkpoint = yielded.inspection().checkpoint().identity().to_owned();
        let resource_attempt = yielded.inspection().yielded_attempt_identity().to_owned();
        let reservations = yielded.inspection().retained_capacity_reservation_count();
        let denied = {
            let fixture_phase_result = {
                let installed_fixture_phase = yielded.fixture_phase_for_test(active_request);
                yielded.readmit_same_runtime(&installed_fixture_phase, &runtime, &bridge)
            };
            match fixture_phase_result {
                worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionOutcome::Denied(denied) => {
                    denied
                }
                _ => panic!("ordinary provider restore failure should deny"),
            }
        };
        assert_eq!(
            denied.kind(),
            worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionDenialKind::ProviderRestoreDenied
        );
        let evidence = denied.readmission_evidence();
        let counters = evidence.query_counters();
        assert_eq!(counters.fresh_resource_attempt_count(), 1);
        assert_eq!(counters.bridge_readmission_attempt_count(), 1);
        assert_eq!(counters.provider_restore_attempt_count(), 1);
        assert_eq!(counters.committed_attempt_count(), 0);
        let bridge_counters = evidence
            .bridge_counters()
            .expect("provider denial must carry final Bridge abort evidence");
        assert_eq!(bridge_counters.abort_count(), 1);
        assert_eq!(bridge_counters.commit_count(), 0);
        let yielded = denied.into_yielded();
        assert_eq!(yielded.inspection().checkpoint().identity(), checkpoint);
        assert_eq!(
            yielded.inspection().yielded_attempt_identity(),
            resource_attempt
        );
        assert_eq!(
            yielded.inspection().retained_capacity_reservation_count(),
            reservations
        );
        let cleanup = complete_direct_yield_cleanup(yielded);
        assert_eq!(cleanup.checkpoint().unwrap().identity(), checkpoint);
    });
}
