use super::super::tests::yield_fixture::YieldProvider;
use super::super::{WorthQueryWorkflowGraphStepOutcome, WorthQueryWorkflowRunCleanupOutcome};

#[test]
fn workflow_checkpoint_failure_and_generation_disruption_become_cleanup_only_authority() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (provider, registry_slot) = YieldProvider::artifact_generation_rollback_failure(7);
        let (yielded, bridge, runtime, _producer) =
            super::super::tests::readmission_workflow::yielded_workflow(execution, provider);
        let generation = yielded
            .inspection()
            .artifact_evidence()
            .production_generation();
        let registry = yielded.artifacts.registry();
        *registry_slot
            .lock()
            .expect("workflow rollback fixture registry slot must remain available") =
            Some(registry);

        let recovery = match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
            crate::domain_computation::WorthQueryWorkflowReadmissionOutcome::RecoveryRequired(
                recovery,
            ) => recovery,
            _ => panic!("coupled checkpoint and generation failure must require recovery"),
        };
        assert_eq!(
        recovery.kind(),
        crate::domain_computation::WorthQueryWorkflowReadmissionRecoveryKind::
            ArtifactGenerationRollbackFailed
    );
        assert_eq!(
        recovery.posture(),
        crate::domain_computation::WorthQueryWorkflowReadmissionRecoveryPosture::
            ArtifactGenerationCleanupRequired
    );
        assert_eq!(
            recovery
                .checkpoint_release()
                .expect("provider checkpoint release evidence must survive")
                .disposition(),
            crate::domain_computation::WorthQueryProviderCheckpointReleaseDisposition::Panicked
        );

        let recovery = match recovery {
        crate::domain_computation::WorthQueryWorkflowReadmissionRecoveryRequired::TerminalCleanup(
            recovery,
        ) => recovery,
        _ => panic!("failed generation rollback must expose only terminal cleanup authority"),
    };
        let cleanup = recovery.into_cleanup();
        let receipt = match cleanup.finish() {
        crate::domain_computation::WorthQueryWorkflowReadmissionCleanupOutcome::RecoveryRequired(
            receipt,
        ) => receipt,
        _ => panic!("typed generation and provider recovery evidence must survive cleanup"),
    };
        let inspection = receipt.inspection();
        let rollback = inspection
            .generation_rollback()
            .expect("cleanup receipt must carry typed generation rollback evidence");
        assert_eq!(rollback.prior_generation(), generation);
        assert_eq!(rollback.pending_generation(), generation + 1);
        assert_eq!(
            rollback.denial_kind(),
            crate::domain_computation::WorthQueryArtifactDenialKind::StaleLifecycleGeneration
        );
        assert_eq!(
            inspection.checkpoint().release_disposition(),
            crate::domain_computation::WorthQueryProviderCheckpointReleaseDisposition::Panicked
        );
        assert!(inspection.resources_released());
        assert_eq!(inspection.released_reservation_count(), 3);
        assert_eq!(inspection.artifact_evidence().retained_artifact_count(), 0);
    });
}

#[test]
fn workflow_generation_mismatch_denies_before_fresh_authority_and_rolls_back_cleanly() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime, _producer) =
            super::super::tests::readmission_workflow::yielded_workflow(
                execution,
                YieldProvider::installed(7),
            );
        let generation = yielded
            .inspection()
            .artifact_evidence()
            .production_generation();
        let pending = yielded
            .artifacts
            .registry()
            .prepare_next_generation()
            .expect("test delta should place the registry in a pending generation");
        let denied = match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
            crate::domain_computation::WorthQueryWorkflowReadmissionOutcome::Denied(denied) => {
                denied
            }
            _ => panic!("non-frozen yielded generation should deny during preflight"),
        };
        assert_eq!(
        denied.kind(),
        crate::domain_computation::WorthQueryWorkflowReadmissionDenialKind::ArtifactGenerationMismatch
    );
        let evidence = denied.readmission_evidence();
        let counters = evidence.query_counters();
        assert_eq!(counters.fresh_resource_attempt_count(), 0);
        assert_eq!(counters.bridge_readmission_attempt_count(), 0);
        assert_eq!(counters.provider_restore_attempt_count(), 0);
        assert!(evidence.bridge_counters().is_none());
        drop(pending);
        let yielded = denied.into_yielded();
        assert_eq!(
            yielded
                .inspection()
                .artifact_evidence()
                .production_generation(),
            generation
        );
        let active = match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
            crate::domain_computation::WorthQueryWorkflowReadmissionOutcome::Readmitted(
                readmitted,
            ) => readmitted.into_active(),
            _ => panic!("generation rollback should restore exact retry authority"),
        };
        let completion = match active.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Completed(completion) => completion,
            _ => panic!("restored workflow provider should complete"),
        };
        match completion.into_running().completed().unwrap().cleanup() {
            WorthQueryWorkflowRunCleanupOutcome::Complete(_) => {}
            _ => panic!("retried workflow should clean up"),
        }
    });
}
