use super::*;

impl WorthQueryWorkflowReadmissionYieldReassembled {
    pub const fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.evidence
    }

    pub fn into_yielded(self) -> WorthQueryYieldedWorkflowRun {
        self.yielded
    }
}

pub fn owner_retry_yielded(
    yielded: WorthQueryYieldedWorkflowRun,
    progress: WorthQueryReadmissionProgress,
) -> WorthQueryWorkflowReadmissionYieldReassemblyOutcome {
    WorthQueryWorkflowReadmissionYieldReassemblyOutcome::Yielded(
        WorthQueryWorkflowReadmissionYieldReassembled {
            yielded,
            evidence: progress.evidence(),
        },
    )
}

pub fn owner_retry_required(
    association: WorthQueryWorkflowBridgeRecoveryAssociation,
    detail: Arc<str>,
    progress: WorthQueryReadmissionProgress,
) -> WorthQueryWorkflowReadmissionYieldReassemblyOutcome {
    WorthQueryWorkflowReadmissionYieldReassemblyOutcome::RecoveryRequired(
        WorthQueryWorkflowReadmissionYieldReassemblyRecovery {
            kind: WorthQueryWorkflowReadmissionRecoveryKind::BridgeCleanupFailed,
            detail,
            progress,
            recovery: WorthQueryWorkflowBridgeCleanupRecoveryState { association },
        },
    )
}
