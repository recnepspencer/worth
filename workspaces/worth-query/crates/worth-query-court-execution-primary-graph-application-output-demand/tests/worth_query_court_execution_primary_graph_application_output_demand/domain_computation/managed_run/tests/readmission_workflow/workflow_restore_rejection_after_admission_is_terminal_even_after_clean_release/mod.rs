//! Workflow restore rejection after admission is terminal even after clean release.

use worth_query_execution::domain_computation::managed_run::readmission::workflow_progression::workflow_state::workflow_cleanup_owner as WorthQueryWorkflowReadmissionCleanupOutcome;

use crate::domain_computation::managed_run::tests::readmission_workflow::*;

#[test]
fn workflow_restore_rejection_after_admission_is_terminal_even_after_clean_release() {
    worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime, _producer) = yielded_workflow(
            execution,
            YieldProvider::checkpoint_restore_reject_after_admission(7),
            resource_request,
        );
        let prior_release_count = yielded
            .inspection()
            .provider_work()
            .provider_execution_release()
            .release_count();
        let recovery = {
            let fixture_phase_result = {
                let installed_fixture_phase = yielded.fixture_phase(active_request);
                yielded.readmit_same_runtime(&installed_fixture_phase, &runtime, &bridge)
            };
            match fixture_phase_result {
            worth_query_execution::facade::domain_computation::WorthQueryWorkflowReadmissionOutcome::RecoveryRequired(
                recovery,
            ) => recovery,
            _ => panic!("post-admission workflow restore rejection became ordinary denial"),
        }
        };
        assert_eq!(
        recovery.kind(),
        worth_query_execution::facade::domain_computation::WorthQueryWorkflowReadmissionRecoveryKind::ProviderRestoreRejectedAfterExecutionAdmission
    );
        let release = recovery
            .restored_execution_release_evidence()
            .expect("workflow recovery must retain replacement release evidence");
        assert!(!release.recovery_required());
        assert_eq!(
        recovery.posture(),
        worth_query_execution::facade::domain_computation::WorthQueryWorkflowReadmissionRecoveryPosture::TerminalCleanupRequired
    );
        let recovery = match recovery {
        worth_query_execution::facade::domain_computation::WorthQueryWorkflowReadmissionRecoveryRequired::TerminalCleanup(
            recovery,
        ) => recovery,
        _ => panic!("post-admission provider rejection must not expose retry authority"),
    };
        match recovery.into_cleanup().finish() {
            WorthQueryWorkflowReadmissionCleanupOutcome::Complete(
                receipt,
            ) => {
                assert_eq!(
                    receipt
                        .inspection()
                        .provider_work()
                        .provider_execution_release()
                        .release_count(),
                    prior_release_count + 1
                );
            }
            _ => panic!("released workflow replacement should complete terminal cleanup"),
        }
    });
}
