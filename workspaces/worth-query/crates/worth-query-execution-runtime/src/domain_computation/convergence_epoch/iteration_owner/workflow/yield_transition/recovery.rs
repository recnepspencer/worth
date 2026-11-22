use super::super::super::super::{
    WorthQueryBoundConvergenceReport, WorthQueryConvergenceEpochCounters,
    WorthQueryRetainedConvergenceCandidateEvidence,
};
use super::super::association::{
    WorkflowAssociatedYieldRecovery, WorkflowAssociatedYieldRecoveryCleanupOutcome,
    WorkflowYieldRecoveryCleanupPendingAssociation, WorkflowYieldRecoveryCleanupReceiptAssociation,
    WorkflowYieldRunningRecoveryAssociation, WorkflowYieldTerminalCleanupAssociation,
};
use super::super::WorthQueryPausedWorkflowConvergenceIteration;

#[must_use = "yield recovery authority must be resumed or cleaned up"]
pub enum WorthQueryWorkflowConvergenceYieldRecoveryRequired {
    RunningAttempt(WorthQueryWorkflowConvergenceYieldRunningRecovery),
    TerminalCleanup(WorthQueryWorkflowConvergenceYieldTerminalCleanupRequired),
}

#[must_use = "running yield recovery must be resumed"]
pub struct WorthQueryWorkflowConvergenceYieldRunningRecovery {
    association: WorkflowYieldRunningRecoveryAssociation,
}

#[must_use = "terminal yield recovery must release its retained resources"]
pub struct WorthQueryWorkflowConvergenceYieldTerminalCleanupRequired {
    association: WorkflowYieldTerminalCleanupAssociation,
}

#[must_use = "workflow yield recovery cleanup must resolve every release posture"]
pub enum WorthQueryWorkflowConvergenceYieldRecoveryCleanupOutcome {
    Complete(WorthQueryWorkflowConvergenceYieldRecoveryCleanupReceipt),
    Pending(WorthQueryWorkflowConvergenceYieldRecoveryCleanupPending),
    RecoveryRequired(WorthQueryWorkflowConvergenceYieldRecoveryCleanupReceipt),
}

#[must_use = "pending workflow yield recovery cleanup retains retry authority"]
pub struct WorthQueryWorkflowConvergenceYieldRecoveryCleanupPending {
    association: WorkflowYieldRecoveryCleanupPendingAssociation,
}

pub struct WorthQueryWorkflowConvergenceYieldRecoveryCleanupReceipt {
    association: WorkflowYieldRecoveryCleanupReceiptAssociation,
}

impl WorthQueryWorkflowConvergenceYieldRunningRecovery {
    pub fn resume(
        self,
    ) -> Result<
        WorthQueryPausedWorkflowConvergenceIteration,
        WorthQueryWorkflowConvergenceYieldTerminalCleanupRequired,
    > {
        self.association
            .resume()
            .map(|association| WorthQueryPausedWorkflowConvergenceIteration { association })
            .map_err(
                |association| WorthQueryWorkflowConvergenceYieldTerminalCleanupRequired {
                    association,
                },
            )
    }
}

impl WorthQueryWorkflowConvergenceYieldTerminalCleanupRequired {
    pub fn finish(
        self,
    ) -> Result<
        WorthQueryWorkflowConvergenceYieldRecoveryCleanupOutcome,
        WorthQueryWorkflowConvergenceYieldRunningRecovery,
    > {
        self.association
            .finish()
            .map(admit_cleanup_outcome)
            .map_err(
                |association| WorthQueryWorkflowConvergenceYieldRunningRecovery { association },
            )
    }
}

impl WorthQueryWorkflowConvergenceYieldRecoveryCleanupPending {
    pub fn identity(&self) -> &str {
        self.association.core().identity()
    }

    pub fn counters(&self) -> &WorthQueryConvergenceEpochCounters {
        self.association.core().counters()
    }

    pub fn retry(
        self,
    ) -> Result<
        WorthQueryWorkflowConvergenceYieldRecoveryCleanupOutcome,
        WorthQueryWorkflowConvergenceYieldRunningRecovery,
    > {
        self.association
            .retry()
            .map(admit_cleanup_outcome)
            .map_err(
                |association| WorthQueryWorkflowConvergenceYieldRunningRecovery { association },
            )
    }
}

impl WorthQueryWorkflowConvergenceYieldRecoveryCleanupReceipt {
    pub fn identity(&self) -> &str {
        self.association.core().identity()
    }

    pub fn counters(&self) -> &WorthQueryConvergenceEpochCounters {
        self.association.core().counters()
    }

    pub fn incumbents(&self) -> &[WorthQueryRetainedConvergenceCandidateEvidence] {
        self.association.core().incumbents()
    }

    pub fn latest_report(&self) -> Option<&WorthQueryBoundConvergenceReport> {
        self.association.core().latest_report()
    }
}

pub(super) fn admit_associated_recovery(
    recovery: WorkflowAssociatedYieldRecovery,
) -> WorthQueryWorkflowConvergenceYieldRecoveryRequired {
    match recovery {
        WorkflowAssociatedYieldRecovery::RunningAttempt(association) => {
            WorthQueryWorkflowConvergenceYieldRecoveryRequired::RunningAttempt(
                WorthQueryWorkflowConvergenceYieldRunningRecovery { association },
            )
        }
        WorkflowAssociatedYieldRecovery::TerminalCleanup(association) => {
            WorthQueryWorkflowConvergenceYieldRecoveryRequired::TerminalCleanup(
                WorthQueryWorkflowConvergenceYieldTerminalCleanupRequired { association },
            )
        }
    }
}

fn admit_cleanup_outcome(
    outcome: WorkflowAssociatedYieldRecoveryCleanupOutcome,
) -> WorthQueryWorkflowConvergenceYieldRecoveryCleanupOutcome {
    match outcome {
        WorkflowAssociatedYieldRecoveryCleanupOutcome::Complete(association) => {
            WorthQueryWorkflowConvergenceYieldRecoveryCleanupOutcome::Complete(
                WorthQueryWorkflowConvergenceYieldRecoveryCleanupReceipt { association },
            )
        }
        WorkflowAssociatedYieldRecoveryCleanupOutcome::Pending(association) => {
            WorthQueryWorkflowConvergenceYieldRecoveryCleanupOutcome::Pending(
                WorthQueryWorkflowConvergenceYieldRecoveryCleanupPending { association },
            )
        }
        WorkflowAssociatedYieldRecoveryCleanupOutcome::RecoveryRequired(association) => {
            WorthQueryWorkflowConvergenceYieldRecoveryCleanupOutcome::RecoveryRequired(
                WorthQueryWorkflowConvergenceYieldRecoveryCleanupReceipt { association },
            )
        }
    }
}
