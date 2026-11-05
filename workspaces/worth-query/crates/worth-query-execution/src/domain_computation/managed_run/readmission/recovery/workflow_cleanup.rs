use crate::domain_computation::vocabulary::managed_run::relational_observation::WorthQueryManagedRelationalObservationReleaseReceipt;
use worth_runtime_bridge::facade::{
    BridgeExecutionBasisFinalizationReceipt, BridgeExecutionBasisReadmissionCleanupOutcome,
    BridgeExecutionBasisReadmissionRecoveryRequired,
};

use super::{
    WorthQueryWorkflowBridgeRecoveryAssociation, WorthQueryWorkflowRestoredAssociation,
    WorthQueryWorkflowYieldedState,
};
use crate::domain_computation::artifact_owner::{
    WorthQueryArtifactProductionGenerationAbortFailure, WorthQueryFrozenWorkflowArtifactAuthority,
    WorthQueryWorkflowArtifactRegistryEvidence,
};
use crate::domain_computation::managed_run::provider_restore::WorthQueryManagedGraphRestoreCleanupRequired;
use crate::domain_computation::managed_run::readmission::evidence::WorthQueryReadmissionProgress;
use crate::domain_computation::managed_run::readmission::workflow_recovery::WorthQueryWorkflowReadmissionRecoveryPermit;
use {crate::domain_computation::vocabulary::managed_run::terminal::WorthQueryManagedRunCleanupDisposition, crate::domain_computation::vocabulary::managed_run::counters::WorthQueryManagedRunCounters, crate::domain_computation::managed_run::WorthQueryYieldTransitionCounters};
use crate::domain_computation::vocabulary::provider_session::provider_execution_release::WorthQueryProviderExecutionReleaseEvidence;
use crate::domain_computation::WorthQueryProviderCheckpointReleaseEvidence;

#[path = "workflow_cleanup/inspection.rs"]
mod inspection;

use inspection::WorthQueryCompletedWorkflowReadmissionCleanup;
pub use inspection::{
    WorthQueryArtifactGenerationRollbackEvidence, WorthQueryWorkflowReadmissionCleanupInspection,
    WorthQueryWorkflowReadmissionCleanupPendingInspection,
};

#[must_use = "workflow readmission cleanup retains its receipt or unfinished cleanup authority"]
pub enum WorthQueryWorkflowReadmissionCleanupOutcome {
    Complete(WorthQueryWorkflowReadmissionCleanupReceipt),
    Pending(WorthQueryWorkflowReadmissionCleanupPending),
    RecoveryRequired(WorthQueryWorkflowReadmissionCleanupReceipt),
}

#[must_use = "workflow readmission cleanup must be finished"]
pub struct WorthQueryWorkflowReadmissionCleanupRequired {
    prepared: WorthQueryWorkflowPreparedCleanup,
    generation_rollback: Option<WorthQueryArtifactProductionGenerationAbortFailure>,
    progress: WorthQueryReadmissionProgress,
}

#[must_use = "pending workflow readmission cleanup retains unfinished owner authority"]
pub struct WorthQueryWorkflowReadmissionCleanupPending {
    partial: WorthQueryWorkflowReadmissionPartialCleanupReceipt,
    artifacts: Option<WorthQueryFrozenWorkflowArtifactAuthority>,
    bridge: Option<BridgeExecutionBasisReadmissionRecoveryRequired>,
    inspection: WorthQueryWorkflowReadmissionCleanupPendingInspection,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryWorkflowReadmissionCleanupReceipt {
    inspection: WorthQueryWorkflowReadmissionCleanupInspection,
}

struct WorthQueryWorkflowReadmissionPartialCleanupReceipt {
    affinity:
        crate::domain_computation::managed_run::workflow::WorthQueryWorkflowAffinityCleanupReceipt,
    checkpoint_release: WorthQueryProviderCheckpointReleaseEvidence,
    restored_execution_release: Option<WorthQueryProviderExecutionReleaseEvidence>,
    bridge: Option<BridgeExecutionBasisFinalizationReceipt>,
    relational: WorthQueryManagedRelationalObservationReleaseReceipt,
    artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
    generation_rollback: Option<WorthQueryArtifactGenerationRollbackEvidence>,
    run_counters: WorthQueryManagedRunCounters,
    yield_counters: WorthQueryYieldTransitionCounters,
    readmission_progress: WorthQueryReadmissionProgress,
}

enum WorthQueryWorkflowCleanupBridge {
    Pending(worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionPending),
    Recovery(BridgeExecutionBasisReadmissionRecoveryRequired),
}

struct WorthQueryWorkflowPreparedCleanup {
    state: WorthQueryWorkflowYieldedState,
    affinity: crate::domain_computation::managed_run::workflow::WorthQueryWorkflowRunAffinity,
    bridge: WorthQueryWorkflowCleanupBridge,
    provider: WorthQueryManagedGraphRestoreCleanupRequired,
}

pub(in crate::domain_computation::managed_run) struct WorthQueryWorkflowReadmissionCleanupPermit {
    _owner: (),
}

impl WorthQueryWorkflowReadmissionCleanupPermit {
    fn mint() -> Self {
        Self { _owner: () }
    }
}

impl WorthQueryWorkflowReadmissionCleanupRequired {
    pub(in crate::domain_computation::managed_run::readmission) fn provider(
        association: WorthQueryWorkflowRestoredAssociation,
        provider: WorthQueryManagedGraphRestoreCleanupRequired,
        generation_rollback: Option<WorthQueryArtifactProductionGenerationAbortFailure>,
        progress: WorthQueryReadmissionProgress,
        _owner: &WorthQueryWorkflowReadmissionRecoveryPermit,
    ) -> Self {
        Self {
            prepared: WorthQueryWorkflowPreparedCleanup::provider(association, provider, _owner),
            generation_rollback,
            progress,
        }
    }

    pub(in crate::domain_computation::managed_run::readmission) fn bridge_recovery(
        association: WorthQueryWorkflowBridgeRecoveryAssociation,
        progress: WorthQueryReadmissionProgress,
        _owner: &WorthQueryWorkflowReadmissionRecoveryPermit,
    ) -> Self {
        Self {
            prepared: WorthQueryWorkflowPreparedCleanup::bridge_recovery(association, _owner),
            generation_rollback: None,
            progress,
        }
    }

    #[must_use = "finishing workflow readmission cleanup returns the exact cleanup posture"]
    pub fn finish(self) -> WorthQueryWorkflowReadmissionCleanupOutcome {
        let owner = WorthQueryWorkflowReadmissionCleanupPermit::mint();
        self.prepared
            .owner_finish(self.generation_rollback, self.progress, &owner)
    }
}

impl WorthQueryWorkflowPreparedCleanup {
    fn provider(
        association: WorthQueryWorkflowRestoredAssociation,
        provider: WorthQueryManagedGraphRestoreCleanupRequired,
        owner: &WorthQueryWorkflowReadmissionRecoveryPermit,
    ) -> Self {
        let WorthQueryWorkflowRestoredAssociation {
            state,
            resource,
            bridge,
        } = association;
        Self {
            state,
            affinity: resource.abort_recovery(owner),
            bridge: WorthQueryWorkflowCleanupBridge::Pending(bridge),
            provider,
        }
    }

    fn bridge_recovery(
        association: WorthQueryWorkflowBridgeRecoveryAssociation,
        _owner: &WorthQueryWorkflowReadmissionRecoveryPermit,
    ) -> Self {
        let WorthQueryWorkflowBridgeRecoveryAssociation {
            state,
            affinity,
            bridge,
            execution,
        } = association;
        Self {
            state,
            affinity,
            bridge: WorthQueryWorkflowCleanupBridge::Recovery(bridge),
            provider: WorthQueryManagedGraphRestoreCleanupRequired::retained(execution, None),
        }
    }

    fn owner_finish(
        mut self,
        generation_rollback: Option<WorthQueryArtifactProductionGenerationAbortFailure>,
        progress: WorthQueryReadmissionProgress,
        owner: &WorthQueryWorkflowReadmissionCleanupPermit,
    ) -> WorthQueryWorkflowReadmissionCleanupOutcome {
        let provider = self.provider.finish();
        if let Some(release) = &provider.restored_execution_release {
            self.affinity.record_provider_execution_release(release);
        }
        let WorthQueryWorkflowYieldedState {
            relational_basis,
            artifacts,
            artifact_evidence: _,
            run_counters,
            provider_artifact_occurrences: _,
            yield_counters,
            inspection: _,
        } = self.state;
        let affinity = self.affinity.finish_cleanup(owner);
        let registry = artifacts.registry();
        registry.close_cancelled();
        let artifact_evidence = registry.evidence();
        let artifacts = (artifact_evidence.retained_artifact_count() != 0
            || artifact_evidence.provider_release_pending_count() != 0)
            .then_some(artifacts);
        let mut partial = WorthQueryWorkflowReadmissionPartialCleanupReceipt {
            affinity,
            checkpoint_release: provider.checkpoint_release,
            restored_execution_release: provider.restored_execution_release,
            bridge: None,
            relational: relational_basis.release(),
            artifact_evidence,
            generation_rollback: generation_rollback
                .map(WorthQueryArtifactGenerationRollbackEvidence::capture),
            run_counters,
            yield_counters,
            readmission_progress: progress,
        };
        let bridge = match self.bridge {
            WorthQueryWorkflowCleanupBridge::Pending(bridge) => bridge.abort(),
            WorthQueryWorkflowCleanupBridge::Recovery(bridge) => bridge.retry_cleanup(),
        };
        let bridge = apply_bridge_cleanup(&mut partial, bridge);
        finish_or_retain_workflow_cleanup(partial, artifacts, bridge)
    }
}

impl WorthQueryWorkflowReadmissionCleanupPending {
    #[must_use = "retrying workflow readmission cleanup returns the exact cleanup posture"]
    pub fn retry(mut self) -> WorthQueryWorkflowReadmissionCleanupOutcome {
        if let Some(artifacts) = self.artifacts.take() {
            let registry = artifacts.registry();
            registry.close_cancelled();
            self.partial.artifact_evidence = registry.evidence();
            if self.partial.artifact_evidence.retained_artifact_count() != 0
                || self
                    .partial
                    .artifact_evidence
                    .provider_release_pending_count()
                    != 0
            {
                self.artifacts = Some(artifacts);
            }
        }
        if let Some(bridge) = self.bridge.take() {
            self.bridge = apply_bridge_cleanup(&mut self.partial, bridge.retry_cleanup());
        }
        finish_or_retain_workflow_cleanup(self.partial, self.artifacts, self.bridge)
    }

    pub const fn inspection(&self) -> &WorthQueryWorkflowReadmissionCleanupPendingInspection {
        &self.inspection
    }
}

fn apply_bridge_cleanup(
    partial: &mut WorthQueryWorkflowReadmissionPartialCleanupReceipt,
    bridge: BridgeExecutionBasisReadmissionCleanupOutcome,
) -> Option<BridgeExecutionBasisReadmissionRecoveryRequired> {
    match bridge {
        BridgeExecutionBasisReadmissionCleanupOutcome::Complete(returned) => {
            let (bridge, bridge_counters) = returned.into_parts();
            partial.readmission_progress.observe_bridge(bridge_counters);
            partial.bridge = Some(bridge.release());
            None
        }
        BridgeExecutionBasisReadmissionCleanupOutcome::RecoveryRequired(bridge) => {
            partial
                .readmission_progress
                .observe_bridge(bridge.counters());
            Some(bridge)
        }
    }
}

fn finish_or_retain_workflow_cleanup(
    partial: WorthQueryWorkflowReadmissionPartialCleanupReceipt,
    artifacts: Option<WorthQueryFrozenWorkflowArtifactAuthority>,
    bridge: Option<BridgeExecutionBasisReadmissionRecoveryRequired>,
) -> WorthQueryWorkflowReadmissionCleanupOutcome {
    if artifacts.is_some() || bridge.is_some() {
        let inspection = WorthQueryWorkflowReadmissionCleanupPendingInspection::capture(
            &partial,
            artifacts.is_some(),
            bridge.is_some(),
        );
        return WorthQueryWorkflowReadmissionCleanupOutcome::Pending(
            WorthQueryWorkflowReadmissionCleanupPending {
                partial,
                artifacts,
                bridge,
                inspection,
            },
        );
    }
    let recovery_required = workflow_cleanup_recovery_required(&partial);
    let disposition = if recovery_required {
        WorthQueryManagedRunCleanupDisposition::RecoveryRequired
    } else {
        WorthQueryManagedRunCleanupDisposition::CleanupComplete
    };
    let bridge = partial
        .bridge
        .expect("completed workflow readmission cleanup has Bridge evidence");
    let receipt = WorthQueryWorkflowReadmissionCleanupReceipt::from_completed(
        WorthQueryCompletedWorkflowReadmissionCleanup {
            affinity: partial.affinity,
            disposition,
            checkpoint_release: partial.checkpoint_release,
            restored_execution_release: partial.restored_execution_release,
            bridge,
            relational: partial.relational,
            artifact_evidence: partial.artifact_evidence,
            generation_rollback: partial.generation_rollback,
            run_counters: partial.run_counters,
            yield_counters: partial.yield_counters,
            readmission_evidence: partial.readmission_progress.evidence(),
        },
    );
    if recovery_required {
        WorthQueryWorkflowReadmissionCleanupOutcome::RecoveryRequired(receipt)
    } else {
        WorthQueryWorkflowReadmissionCleanupOutcome::Complete(receipt)
    }
}

fn workflow_cleanup_recovery_required(
    receipt: &WorthQueryWorkflowReadmissionPartialCleanupReceipt,
) -> bool {
    receipt.checkpoint_release.disposition().recovery_required()
        || receipt
            .restored_execution_release
            .as_ref()
            .is_some_and(WorthQueryProviderExecutionReleaseEvidence::recovery_required)
        || receipt
            .artifact_evidence
            .provider_release_recovery_required_count()
            != 0
        || receipt.generation_rollback.is_some()
}
