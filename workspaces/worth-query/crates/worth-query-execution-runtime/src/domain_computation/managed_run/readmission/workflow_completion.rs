use std::sync::Arc;

use worth_runtime_bridge::facade::RuntimeBridge;

use super::super::super::provider_restore::{
    WorthQueryManagedGraphRestoreCommitOutcome, WorthQueryManagedGraphRestorePending,
};
use super::super::super::WorthQueryActiveWorkflowGraphExecution;
use super::super::evidence::WorthQueryReadmissionProgress;
use super::super::workflow_outcome::{
    WorthQueryWorkflowReadmissionDenialKind, WorthQueryWorkflowReadmissionOutcome,
};
use super::super::workflow_recovery::WorthQueryWorkflowReadmissionRecoveryRequired;
use super::workflow_abort::{abort_provider_pending, map_recovery_kind};
use super::workflow_state::{
    WorthQueryWorkflowCommittedAssociation, WorthQueryWorkflowRestoredAssociation,
};
use super::WorthQueryWorkflowReadmissionProgressionPermit;
use crate::domain_computation::artifact_owner::WorthQueryArtifactProductionGenerationPending;

pub(super) fn advance_artifact_generation(
    association: WorthQueryWorkflowRestoredAssociation,
    stage_identity: String,
    provider: WorthQueryManagedGraphRestorePending,
    bridge_runtime: &RuntimeBridge,
    mut progress: WorthQueryReadmissionProgress,
    owner: &WorthQueryWorkflowReadmissionProgressionPermit,
) -> WorthQueryWorkflowReadmissionOutcome {
    progress.attempted_artifact_generation();
    let generation = match association.owner_prepare_artifact_generation(owner) {
        Ok(generation) => generation,
        Err(denial) => {
            return abort_provider_pending(
                WorthQueryWorkflowReadmissionDenialKind::ArtifactGenerationDenied,
                denial.detail(),
                association,
                provider,
                progress,
                owner,
            );
        }
    };
    let artifact_context =
        match association.owner_artifact_context(&stage_identity, &generation, owner) {
            Ok(context) => context,
            Err(denial) => {
                let detail = Arc::from(denial.detail());
                if let Err(generation_rollback) = generation.abort() {
                    return WorthQueryWorkflowReadmissionOutcome::RecoveryRequired(
                        WorthQueryWorkflowReadmissionRecoveryRequired::provider_pending(
                            format!(
                                "{detail}; generation rollback failed: {}",
                                generation_rollback.detail()
                            ),
                            progress,
                            association,
                            provider,
                            generation_rollback,
                            owner,
                        ),
                    );
                }
                return abort_provider_pending(
                    WorthQueryWorkflowReadmissionDenialKind::ArtifactAuthorityDenied,
                    detail,
                    association,
                    provider,
                    progress,
                    owner,
                );
            }
        };
    commit_artifact_generation(
        association,
        provider,
        generation,
        artifact_context,
        bridge_runtime,
        progress,
        owner,
    )
}

fn commit_artifact_generation(
    association: WorthQueryWorkflowRestoredAssociation,
    provider: WorthQueryManagedGraphRestorePending,
    generation: WorthQueryArtifactProductionGenerationPending,
    artifact_context: Option<
        crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryGraphProviderStepArtifactContext,
    >,
    bridge_runtime: &RuntimeBridge,
    mut progress: WorthQueryReadmissionProgress,
    owner: &WorthQueryWorkflowReadmissionProgressionPermit,
) -> WorthQueryWorkflowReadmissionOutcome {
    let execution = match provider.commit(artifact_context) {
        WorthQueryManagedGraphRestoreCommitOutcome::Restored(execution) => execution,
        WorthQueryManagedGraphRestoreCommitOutcome::RecoveryRequired(recovery) => {
            let kind = map_recovery_kind(recovery.kind());
            let mut detail = recovery.detail().to_owned();
            if let Err(generation_rollback) = generation.abort() {
                detail.push_str("; generation rollback failed: ");
                detail.push_str(generation_rollback.detail());
                return WorthQueryWorkflowReadmissionOutcome::RecoveryRequired(
                    WorthQueryWorkflowReadmissionRecoveryRequired::provider_generation(
                        detail,
                        progress,
                        association,
                        recovery,
                        generation_rollback,
                        owner,
                    ),
                );
            }
            return WorthQueryWorkflowReadmissionOutcome::RecoveryRequired(
                WorthQueryWorkflowReadmissionRecoveryRequired::provider(
                    kind,
                    detail,
                    progress,
                    association,
                    recovery,
                    owner,
                ),
            );
        }
    };
    let committed_generation = generation.commit();
    progress.committed_artifact_generation();
    commit_workflow(
        association.owner_commit_generation(committed_generation, owner),
        execution,
        bridge_runtime,
        progress,
        owner,
    )
}

fn commit_workflow(
    state: WorthQueryWorkflowCommittedAssociation,
    execution: super::super::super::managed_graph_execution::WorthQueryManagedGraphExecution,
    bridge_runtime: &RuntimeBridge,
    mut progress: WorthQueryReadmissionProgress,
    owner: &WorthQueryWorkflowReadmissionProgressionPermit,
) -> WorthQueryWorkflowReadmissionOutcome {
    let (running, bridge_counters) = state.owner_commit(bridge_runtime, owner);
    progress.observe_bridge(bridge_counters);
    progress.committed_attempt();
    let active = WorthQueryActiveWorkflowGraphExecution::new(running, execution);
    WorthQueryWorkflowReadmissionOutcome::Readmitted(
        super::super::WorthQueryReadmittedWorkflowGraphExecution::new(active, progress.evidence()),
    )
}
