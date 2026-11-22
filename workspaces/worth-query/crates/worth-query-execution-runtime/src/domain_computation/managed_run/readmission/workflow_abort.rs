use std::sync::Arc;

use super::super::super::provider_restore::WorthQueryManagedGraphRestorePending;
use super::super::super::provider_restore::{
    WorthQueryManagedGraphRestoreAbortOutcome, WorthQueryManagedGraphRestoreRecoveryKind,
};
use super::super::super::retained_graph_execution::WorthQueryRetainedManagedGraphExecution;
use super::super::evidence::WorthQueryReadmissionProgress;
use super::super::workflow_outcome::{
    WorthQueryWorkflowReadmissionDenialKind, WorthQueryWorkflowReadmissionOutcome,
};
use super::super::workflow_recovery::{
    WorthQueryWorkflowReadmissionRecoveryKind, WorthQueryWorkflowReadmissionRecoveryRequired,
};
use super::workflow_state::WorthQueryWorkflowRestoredAssociation;
use super::WorthQueryWorkflowReadmissionProgressionPermit;

pub(super) fn abort_without_provider(
    kind: WorthQueryWorkflowReadmissionDenialKind,
    detail: impl Into<Arc<str>>,
    association: WorthQueryWorkflowRestoredAssociation,
    execution: WorthQueryRetainedManagedGraphExecution,
    progress: WorthQueryReadmissionProgress,
    owner: &WorthQueryWorkflowReadmissionProgressionPermit,
) -> WorthQueryWorkflowReadmissionOutcome {
    association
        .owner_abort_bridge(execution, owner)
        .owner_resolve_denial(kind, detail.into(), progress, owner)
}

pub(super) fn abort_provider_pending(
    kind: WorthQueryWorkflowReadmissionDenialKind,
    detail: impl Into<Arc<str>>,
    association: WorthQueryWorkflowRestoredAssociation,
    provider: WorthQueryManagedGraphRestorePending,
    progress: WorthQueryReadmissionProgress,
    owner: &WorthQueryWorkflowReadmissionProgressionPermit,
) -> WorthQueryWorkflowReadmissionOutcome {
    let detail = detail.into();
    match provider.abort() {
        WorthQueryManagedGraphRestoreAbortOutcome::Aborted(execution) => {
            abort_without_provider(kind, detail, association, execution, progress, owner)
        }
        WorthQueryManagedGraphRestoreAbortOutcome::RecoveryRequired(recovery) => {
            let recovery_kind = map_recovery_kind(recovery.kind());
            WorthQueryWorkflowReadmissionOutcome::RecoveryRequired(
                WorthQueryWorkflowReadmissionRecoveryRequired::provider(
                    recovery_kind,
                    format!("{detail}; {}", recovery.detail()),
                    progress,
                    association,
                    recovery,
                    owner,
                ),
            )
        }
    }
}

pub(super) fn map_recovery_kind(
    kind: WorthQueryManagedGraphRestoreRecoveryKind,
) -> WorthQueryWorkflowReadmissionRecoveryKind {
    match kind {
        WorthQueryManagedGraphRestoreRecoveryKind::ProviderRestorePanicked => {
            WorthQueryWorkflowReadmissionRecoveryKind::ProviderRestorePanicked
        }
        WorthQueryManagedGraphRestoreRecoveryKind::
            ProviderRestoreRejectedAfterExecutionAdmission => {
                WorthQueryWorkflowReadmissionRecoveryKind::
                    ProviderRestoreRejectedAfterExecutionAdmission
            }
        WorthQueryManagedGraphRestoreRecoveryKind::RestoredExecutionReleaseRecoveryRequired => {
            WorthQueryWorkflowReadmissionRecoveryKind::RestoredExecutionReleaseRecoveryRequired
        }
        WorthQueryManagedGraphRestoreRecoveryKind::CheckpointReleasePanicked => {
            WorthQueryWorkflowReadmissionRecoveryKind::CheckpointReleasePanicked
        }
    }
}
