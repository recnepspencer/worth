mod inspection;

use super::WorthQueryManagedRelationalObservation;
use worth_runtime_bridge::facade::BridgeExecutionBasisFinalizationReceipt;

pub use inspection::{
    WorthQueryWorkflowYieldCleanupInspection, WorthQueryWorkflowYieldCleanupReceipt,
};

use self::inspection::WorthQueryCompletedWorkflowYieldCleanup;
use super::{
    workflow::WorthQueryWorkflowYieldReleasePending, WorthQueryManagedRunCleanupDisposition,
    WorthQueryManagedRunCounters, WorthQueryYieldTransitionCounters, WorthQueryYieldedWorkflowRun,
};
use crate::domain_computation::artifact_owner::{
    WorthQueryFrozenWorkflowArtifactAuthority, WorthQueryWorkflowArtifactRegistryEvidence,
};
use crate::domain_computation::WorthQueryProviderCheckpointReleaseEvidence;

pub(super) struct WorthQueryWorkflowYieldCleanupPermit {
    _owner: (),
}

impl WorthQueryWorkflowYieldCleanupPermit {
    fn mint() -> Self {
        Self { _owner: () }
    }
}

#[must_use = "workflow yielded-run cleanup outcomes must be resolved"]
pub enum WorthQueryWorkflowYieldCleanupOutcome {
    Complete(WorthQueryWorkflowYieldCleanupReceipt),
    Pending(WorthQueryWorkflowYieldCleanupPending),
    RecoveryRequired(WorthQueryWorkflowYieldCleanupReceipt),
}

#[must_use = "pending workflow yielded-run cleanup retains artifact and release authority"]
pub struct WorthQueryWorkflowYieldCleanupPending {
    association: WorthQueryWorkflowYieldCleanupAssociation,
}

struct WorthQueryWorkflowYieldCleanupAssociation {
    affinity: WorthQueryWorkflowYieldReleasePending,
    relational_basis: WorthQueryManagedRelationalObservation,
    bridge: BridgeExecutionBasisFinalizationReceipt,
    artifacts: WorthQueryFrozenWorkflowArtifactAuthority,
    checkpoint_release: WorthQueryProviderCheckpointReleaseEvidence,
    artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
    run_counters: WorthQueryManagedRunCounters,
    yield_counters: WorthQueryYieldTransitionCounters,
}

impl WorthQueryWorkflowYieldCleanupPending {
    #[must_use = "retry returns the same workflow yielded-run cleanup authority"]
    pub fn retry(self) -> WorthQueryWorkflowYieldCleanupOutcome {
        cleanup_without_checkpoint_owner(self.association)
    }
}

pub(super) fn cleanup_yielded_workflow(
    yielded: WorthQueryYieldedWorkflowRun,
) -> WorthQueryWorkflowYieldCleanupOutcome {
    let permit = WorthQueryWorkflowYieldCleanupPermit::mint();
    let association = yielded.owner_into_cleanup_association(&permit);
    let (
        affinity,
        relational_basis,
        bridge,
        artifacts,
        checkpoint_release,
        artifact_evidence,
        run_counters,
        yield_counters,
    ) = association.owner_into_parts(&permit);
    cleanup_without_checkpoint_owner(WorthQueryWorkflowYieldCleanupAssociation {
        affinity,
        relational_basis,
        bridge,
        artifacts,
        checkpoint_release,
        artifact_evidence,
        run_counters,
        yield_counters,
    })
}

fn cleanup_without_checkpoint_owner(
    mut association: WorthQueryWorkflowYieldCleanupAssociation,
) -> WorthQueryWorkflowYieldCleanupOutcome {
    let registry = association.artifacts.registry();
    registry.close_cancelled();
    association.artifact_evidence = registry.evidence();
    if cleanup_remains_pending(association.artifact_evidence) {
        return WorthQueryWorkflowYieldCleanupOutcome::Pending(
            WorthQueryWorkflowYieldCleanupPending { association },
        );
    }
    let recovery_required = association
        .checkpoint_release
        .disposition()
        .recovery_required()
        || association
            .artifact_evidence
            .provider_release_recovery_required_count()
            != 0;
    let receipt = complete_cleanup(association, recovery_required);
    if recovery_required {
        WorthQueryWorkflowYieldCleanupOutcome::RecoveryRequired(receipt)
    } else {
        WorthQueryWorkflowYieldCleanupOutcome::Complete(receipt)
    }
}

fn cleanup_remains_pending(evidence: WorthQueryWorkflowArtifactRegistryEvidence) -> bool {
    evidence.retained_artifact_count() != 0 || evidence.provider_release_pending_count() != 0
}

fn complete_cleanup(
    association: WorthQueryWorkflowYieldCleanupAssociation,
    recovery_required: bool,
) -> WorthQueryWorkflowYieldCleanupReceipt {
    let WorthQueryWorkflowYieldCleanupAssociation {
        affinity,
        relational_basis,
        bridge,
        artifacts,
        checkpoint_release,
        artifact_evidence,
        run_counters,
        yield_counters,
    } = association;
    drop(artifacts);
    WorthQueryWorkflowYieldCleanupReceipt::from_completed(WorthQueryCompletedWorkflowYieldCleanup {
        affinity: affinity.release(),
        disposition: if recovery_required {
            WorthQueryManagedRunCleanupDisposition::RecoveryRequired
        } else {
            WorthQueryManagedRunCleanupDisposition::CleanupComplete
        },
        checkpoint_release,
        bridge,
        relational: relational_basis.release(),
        artifact_evidence,
        run_counters,
        yield_counters,
    })
}
