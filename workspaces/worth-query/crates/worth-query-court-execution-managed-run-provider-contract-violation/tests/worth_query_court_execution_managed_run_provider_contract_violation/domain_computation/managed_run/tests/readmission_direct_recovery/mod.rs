use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::readmission_direct::yielded_direct_with_provider;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::yield_fixture::YieldProvider;

#[test]
fn provider_restore_panic_remains_typed_recovery_with_retained_authority() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime) = yielded_direct_with_provider(
            execution,
            YieldProvider::checkpoint_restore_panic(7),
            resource_request,
        );
        let recovery = {
            let fixture_phase_result = {
                let fixture_target = yielded;
                let installed_fixture_phase = fixture_target.fixture_phase(active_request);
                fixture_target.readmit_same_runtime(&installed_fixture_phase, &runtime, &bridge)
            };
            match fixture_phase_result {
                worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionOutcome::RecoveryRequired(
                    recovery,
                ) => recovery,
                _ => panic!("provider restore panic must not become ordinary denial"),
            }
        };
        assert_eq!(
        recovery.kind(),
        worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionRecoveryKind::ProviderRestorePanicked
    );
        let counters = recovery.readmission_evidence().query_counters();
        assert_eq!(counters.provider_restore_attempt_count(), 1);
        assert_eq!(counters.committed_attempt_count(), 0);
        assert_eq!(
        recovery.posture(),
        worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionRecoveryPosture::TerminalCleanupRequired
    );
        assert!(recovery.checkpoint_release().is_none());
        assert!(recovery.restored_execution_release_evidence().is_none());
        let recovery = match recovery {
        worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionRecoveryRequired::TerminalCleanup(
            recovery,
        ) => recovery,
        _ => panic!("provider panic must not expose yield-reassembly authority"),
    };
        let receipt = match recovery.into_cleanup().finish() {
            worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionCleanupOutcome::Complete(
                receipt,
            ) => receipt,
            _ => panic!("retained checkpoint should release through terminal cleanup"),
        };
        let inspection = receipt.inspection();
        assert_eq!(inspection.checkpoint().retained_bytes(), 7);
        assert!(inspection.resources_released());
    });
}

#[test]
fn provider_restore_rejection_after_admission_carries_exact_release_evidence() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime) = yielded_direct_with_provider(
            execution,
            YieldProvider::checkpoint_restore_reject_after_admission(7),
            resource_request,
        );
        let recovery = {
            let fixture_phase_result = {
                let fixture_target = yielded;
                let installed_fixture_phase = fixture_target.fixture_phase(active_request);
                fixture_target.readmit_same_runtime(&installed_fixture_phase, &runtime, &bridge)
            };
            match fixture_phase_result {
                worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionOutcome::RecoveryRequired(
                    recovery,
                ) => recovery,
                _ => panic!("post-admission restore rejection became ordinary retryable denial"),
            }
        };
        assert_eq!(
        recovery.kind(),
        worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionRecoveryKind::ProviderRestoreRejectedAfterExecutionAdmission
    );
        let release = recovery
            .restored_execution_release_evidence()
            .expect("rejected admitted restore must carry physical-release evidence");
        assert_eq!(
            release.disposal(),
            worth_query_execution::facade::domain_computation::WorthQueryProviderExecutionDisposalDisposition::Completed
        );
        assert_eq!(
            release.destructor(),
            worth_query_execution::facade::domain_computation::WorthQueryProviderExecutionDestructorDisposition::Completed
        );
        assert_eq!(
        recovery.posture(),
        worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionRecoveryPosture::TerminalCleanupRequired
    );
        let recovery = match recovery {
        worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionRecoveryRequired::TerminalCleanup(
            recovery,
        ) => recovery,
        _ => panic!("post-admission provider rejection must not expose retry authority"),
    };
        match recovery.into_cleanup().finish() {
            worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionCleanupOutcome::Complete(
                receipt,
            ) => {
                let inspection = receipt.inspection();
                assert_eq!(
                inspection.checkpoint().release_disposition(),
                worth_query_execution::facade::domain_computation::WorthQueryProviderCheckpointReleaseDisposition::Released
            );
                assert!(!inspection
                    .restored_execution()
                    .expect("replacement release evidence must survive cleanup")
                    .recovery_required());
            }
            _ => panic!("released replacement and checkpoint should complete terminal cleanup"),
        }
    });
}

#[test]
fn restore_panic_and_replacement_destructor_panic_flow_into_terminal_cleanup() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime) = yielded_direct_with_provider(
            execution,
            YieldProvider::checkpoint_restore_panic_after_admission(7),
            resource_request,
        );
        let recovery = {
            let fixture_phase_result = {
                let fixture_target = yielded;
                let installed_fixture_phase = fixture_target.fixture_phase(active_request);
                fixture_target.readmit_same_runtime(&installed_fixture_phase, &runtime, &bridge)
            };
            match fixture_phase_result {
                worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionOutcome::RecoveryRequired(
                    recovery,
                ) => recovery,
                _ => panic!("double restore failure escaped typed recovery"),
            }
        };
        assert_eq!(
        recovery.kind(),
        worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionRecoveryKind::ProviderRestorePanicked
    );
        let release = recovery
            .restored_execution_release_evidence()
            .expect("restore panic must retain replacement release evidence");
        assert_eq!(
            release.disposal(),
            worth_query_execution::facade::domain_computation::WorthQueryProviderExecutionDisposalDisposition::Completed
        );
        assert_eq!(
            release.destructor(),
            worth_query_execution::facade::domain_computation::WorthQueryProviderExecutionDestructorDisposition::Panicked
        );
        assert_eq!(
        recovery.posture(),
        worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionRecoveryPosture::TerminalCleanupRequired
    );
        let recovery = match recovery {
        worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionRecoveryRequired::TerminalCleanup(
            recovery,
        ) => recovery,
        _ => panic!("provider panic must not expose yield-reassembly authority"),
    };
        let receipt = match recovery.into_cleanup().finish() {
        worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionCleanupOutcome::RecoveryRequired(
            receipt,
        ) => receipt,
        _ => panic!("destructor uncertainty must remain visible after terminal cleanup"),
    };
        let carried_release = receipt
            .inspection()
            .provider_work()
            .provider_execution_release()
            .recovery_evidence()
            .expect("cleanup evidence must retain replacement release uncertainty");
        assert_eq!(
            carried_release.destructor(),
            worth_query_execution::facade::domain_computation::WorthQueryProviderExecutionDestructorDisposition::Panicked
        );
    });
}
