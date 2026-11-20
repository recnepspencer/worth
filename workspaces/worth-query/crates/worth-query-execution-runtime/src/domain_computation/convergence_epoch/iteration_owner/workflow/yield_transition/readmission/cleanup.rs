//! Closed workflow convergence semantics for readmission cleanup.

use super::super::super::association::{
    WorkflowAssociatedReadmissionCleanupOutcome, WorkflowReadmissionCleanupPendingAssociation,
    WorkflowReadmissionCleanupReceiptAssociation, WorkflowReadmissionCleanupRequiredAssociation,
};
use crate::domain_computation::{
    WorthQueryBoundConvergenceReport, WorthQueryConvergenceEpochCounters,
    WorthQueryManagedRunCleanupDisposition, WorthQueryReadmissionEvidence,
    WorthQueryRetainedConvergenceCandidateEvidence,
};

#[must_use = "workflow convergence readmission cleanup must be finished"]
pub struct WorthQueryWorkflowConvergenceReadmissionCleanupRequired {
    association: WorkflowReadmissionCleanupRequiredAssociation,
}

#[must_use = "workflow convergence readmission cleanup outcome retains closed or retry authority"]
pub enum WorthQueryWorkflowConvergenceReadmissionCleanupOutcome {
    Complete(WorthQueryWorkflowConvergenceReadmissionCleanupReceipt),
    Pending(WorthQueryWorkflowConvergenceReadmissionCleanupPending),
    RecoveryRequired(WorthQueryWorkflowConvergenceReadmissionCleanupReceipt),
}

#[must_use = "pending workflow convergence readmission cleanup must be retried"]
pub struct WorthQueryWorkflowConvergenceReadmissionCleanupPending {
    association: WorkflowReadmissionCleanupPendingAssociation,
}

pub struct WorthQueryWorkflowConvergenceReadmissionCleanupReceipt {
    association: WorkflowReadmissionCleanupReceiptAssociation,
}

impl WorthQueryWorkflowConvergenceReadmissionCleanupRequired {
    pub(super) fn from_association(
        association: WorkflowReadmissionCleanupRequiredAssociation,
    ) -> Self {
        Self { association }
    }

    pub fn finish(self) -> WorthQueryWorkflowConvergenceReadmissionCleanupOutcome {
        admit_cleanup_outcome(self.association.finish())
    }
}

impl WorthQueryWorkflowConvergenceReadmissionCleanupOutcome {
    pub fn disposition(&self) -> WorthQueryManagedRunCleanupDisposition {
        match self {
            Self::Complete(receipt) | Self::RecoveryRequired(receipt) => receipt.disposition(),
            Self::Pending(_) => WorthQueryManagedRunCleanupDisposition::CleanupPending,
        }
    }

    pub fn counters(&self) -> &WorthQueryConvergenceEpochCounters {
        match self {
            Self::Complete(receipt) | Self::RecoveryRequired(receipt) => receipt.counters(),
            Self::Pending(pending) => pending.counters(),
        }
    }
}

impl WorthQueryWorkflowConvergenceReadmissionCleanupPending {
    pub fn retry(self) -> WorthQueryWorkflowConvergenceReadmissionCleanupOutcome {
        admit_cleanup_outcome(self.association.retry())
    }

    pub fn epoch_identity(&self) -> &str {
        self.association.epoch_identity()
    }

    pub fn counters(&self) -> &WorthQueryConvergenceEpochCounters {
        self.association.counters()
    }

    pub fn incumbents(&self) -> &[WorthQueryRetainedConvergenceCandidateEvidence] {
        self.association.incumbents()
    }

    pub fn latest_report(&self) -> Option<&WorthQueryBoundConvergenceReport> {
        self.association.latest_report()
    }

    pub fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.association.readmission_evidence()
    }
}

impl WorthQueryWorkflowConvergenceReadmissionCleanupReceipt {
    pub fn epoch_identity(&self) -> &str {
        self.association.epoch_identity()
    }

    pub fn counters(&self) -> &WorthQueryConvergenceEpochCounters {
        self.association.counters()
    }

    pub fn incumbents(&self) -> &[WorthQueryRetainedConvergenceCandidateEvidence] {
        self.association.incumbents()
    }

    pub fn latest_report(&self) -> Option<&WorthQueryBoundConvergenceReport> {
        self.association.latest_report()
    }

    pub const fn disposition(&self) -> WorthQueryManagedRunCleanupDisposition {
        self.association.disposition()
    }

    pub const fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.association.readmission_evidence()
    }
}

fn admit_cleanup_outcome(
    outcome: WorkflowAssociatedReadmissionCleanupOutcome,
) -> WorthQueryWorkflowConvergenceReadmissionCleanupOutcome {
    match outcome {
        WorkflowAssociatedReadmissionCleanupOutcome::Complete(association) => {
            WorthQueryWorkflowConvergenceReadmissionCleanupOutcome::Complete(
                WorthQueryWorkflowConvergenceReadmissionCleanupReceipt { association },
            )
        }
        WorkflowAssociatedReadmissionCleanupOutcome::Pending(association) => {
            WorthQueryWorkflowConvergenceReadmissionCleanupOutcome::Pending(
                WorthQueryWorkflowConvergenceReadmissionCleanupPending { association },
            )
        }
        WorkflowAssociatedReadmissionCleanupOutcome::RecoveryRequired(association) => {
            WorthQueryWorkflowConvergenceReadmissionCleanupOutcome::RecoveryRequired(
                WorthQueryWorkflowConvergenceReadmissionCleanupReceipt { association },
            )
        }
    }
}
