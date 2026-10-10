use super::yield_fixture::YieldProvider;

#[test]
fn checkpoint_release_panic_reports_exact_non_retryable_physical_posture() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime) = super::readmission_direct::yielded_direct_with_provider(
            execution,
            YieldProvider::checkpoint_drop_panic(),
            resource_request,
        );
        let checkpoint = yielded.inspection().checkpoint().identity().to_owned();
        let recovery = match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
            crate::domain_computation::WorthQueryDirectReadmissionOutcome::RecoveryRequired(
                recovery,
            ) => recovery,
            _ => panic!("checkpoint release panic must require recovery"),
        };
        assert_eq!(
        recovery.kind(),
        crate::domain_computation::WorthQueryDirectReadmissionRecoveryKind::CheckpointReleasePanicked
    );
        assert_eq!(recovery.checkpoint().identity(), checkpoint);
        assert_eq!(
        recovery.posture(),
        crate::domain_computation::WorthQueryDirectReadmissionRecoveryPosture::
            TerminalCleanupRequired
    );
        assert_eq!(
            recovery.checkpoint_release().unwrap().disposition(),
            crate::domain_computation::WorthQueryProviderCheckpointReleaseDisposition::Panicked
        );
        let restored_release = recovery
            .restored_execution_release_evidence()
            .expect("replacement execution was physically released");
        assert_eq!(
            restored_release.disposal(),
            crate::domain_computation::WorthQueryProviderExecutionDisposalDisposition::Completed
        );
        assert_eq!(
            restored_release.destructor(),
            crate::domain_computation::WorthQueryProviderExecutionDestructorDisposition::Completed
        );
        let recovery = match recovery {
        crate::domain_computation::WorthQueryDirectReadmissionRecoveryRequired::TerminalCleanup(
            recovery,
        ) => recovery,
        _ => panic!("released checkpoint recovery must expose only terminal cleanup authority"),
    };
        let cleanup = recovery.into_cleanup();
        let receipt = match cleanup.finish() {
        crate::domain_computation::WorthQueryDirectReadmissionCleanupOutcome::RecoveryRequired(
            receipt,
        ) => receipt,
        _ => panic!("checkpoint release panic must remain visible after terminal cleanup"),
    };
        let inspection = receipt.inspection();
        assert_eq!(
            inspection.checkpoint().release_disposition(),
            crate::domain_computation::WorthQueryProviderCheckpointReleaseDisposition::Panicked
        );
        assert!(inspection.resources_released());
        assert_eq!(inspection.released_reservation_count(), 2);
    });
}

#[test]
fn checkpoint_and_restored_execution_drop_panics_preserve_both_physical_dispositions() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime) = super::readmission_direct::yielded_direct_with_provider(
            execution,
            YieldProvider::checkpoint_and_restored_execution_drop_panic(7),
            resource_request,
        );
        let recovery = match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
            crate::domain_computation::WorthQueryDirectReadmissionOutcome::RecoveryRequired(
                recovery,
            ) => recovery,
            _ => panic!("causal checkpoint and replacement cleanup panics must require recovery"),
        };
        assert_eq!(
        recovery.kind(),
        crate::domain_computation::WorthQueryDirectReadmissionRecoveryKind::CheckpointReleasePanicked
    );
        assert_eq!(
        recovery.posture(),
        crate::domain_computation::WorthQueryDirectReadmissionRecoveryPosture::
            TerminalCleanupRequired
    );
        assert_eq!(
            recovery
                .checkpoint_release()
                .expect("released checkpoint must expose its disposition")
                .disposition(),
            crate::domain_computation::WorthQueryProviderCheckpointReleaseDisposition::Panicked
        );
        let restored_release = recovery
            .restored_execution_release_evidence()
            .expect("replacement execution release evidence must remain available");
        assert_eq!(
            restored_release.disposal(),
            crate::domain_computation::WorthQueryProviderExecutionDisposalDisposition::Completed
        );
        assert_eq!(
            restored_release.destructor(),
            crate::domain_computation::WorthQueryProviderExecutionDestructorDisposition::Panicked
        );
        let recovery = match recovery {
        crate::domain_computation::WorthQueryDirectReadmissionRecoveryRequired::TerminalCleanup(
            recovery,
        ) => recovery,
        _ => panic!("physical uncertainty must expose only terminal cleanup authority"),
    };
        let cleanup = recovery.into_cleanup();
        let receipt = match cleanup.finish() {
        crate::domain_computation::WorthQueryDirectReadmissionCleanupOutcome::RecoveryRequired(
            receipt,
        ) => receipt,
        _ => panic!("both physical failures must survive complete terminal cleanup"),
    };
        let inspection = receipt.inspection();
        assert_eq!(
            inspection.checkpoint().release_disposition(),
            crate::domain_computation::WorthQueryProviderCheckpointReleaseDisposition::Panicked
        );
        assert!(inspection
            .restored_execution()
            .expect("restored execution release evidence must remain attached")
            .recovery_required());
        assert!(inspection.resources_released());
        assert_eq!(inspection.released_reservation_count(), 2);
    });
}
