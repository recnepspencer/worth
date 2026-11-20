//! Public workflow convergence cleanup phases after a yielded iteration.

use super::super::super::super::{
    WorthQueryBoundConvergenceReport, WorthQueryConvergenceEpochCounters,
    WorthQueryRetainedConvergenceCandidateEvidence,
};
use super::super::association::{
    WorkflowAssociatedYieldCleanupOutcome, WorkflowYieldCleanupPendingAssociation,
    WorkflowYieldCleanupReceiptAssociation,
};

#[must_use = "convergence yield cleanup carries complete, pending, or recovery authority"]
pub enum WorthQueryWorkflowConvergenceYieldCleanupOutcome {
    Complete(WorthQueryWorkflowConvergenceYieldCleanupReceipt),
    Pending(WorthQueryWorkflowConvergenceYieldCleanupPending),
    RecoveryRequired(WorthQueryWorkflowConvergenceYieldCleanupReceipt),
}

#[must_use = "pending convergence yield cleanup must be retried after artifact owners close"]
pub struct WorthQueryWorkflowConvergenceYieldCleanupPending {
    association: WorkflowYieldCleanupPendingAssociation,
}

pub struct WorthQueryWorkflowConvergenceYieldCleanupReceipt {
    association: WorkflowYieldCleanupReceiptAssociation,
}

impl WorthQueryWorkflowConvergenceYieldCleanupPending {
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

    pub fn retry(self) -> WorthQueryWorkflowConvergenceYieldCleanupOutcome {
        admit_associated_cleanup(self.association.retry())
    }
}

impl WorthQueryWorkflowConvergenceYieldCleanupReceipt {
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

pub(super) fn admit_associated_cleanup(
    outcome: WorkflowAssociatedYieldCleanupOutcome,
) -> WorthQueryWorkflowConvergenceYieldCleanupOutcome {
    match outcome {
        WorkflowAssociatedYieldCleanupOutcome::Complete(association) => {
            WorthQueryWorkflowConvergenceYieldCleanupOutcome::Complete(
                WorthQueryWorkflowConvergenceYieldCleanupReceipt { association },
            )
        }
        WorkflowAssociatedYieldCleanupOutcome::Pending(association) => {
            WorthQueryWorkflowConvergenceYieldCleanupOutcome::Pending(
                WorthQueryWorkflowConvergenceYieldCleanupPending { association },
            )
        }
        WorkflowAssociatedYieldCleanupOutcome::RecoveryRequired(association) => {
            WorthQueryWorkflowConvergenceYieldCleanupOutcome::RecoveryRequired(
                WorthQueryWorkflowConvergenceYieldCleanupReceipt { association },
            )
        }
    }
}
