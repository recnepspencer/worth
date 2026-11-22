//! Closed direct convergence semantics for readmission cleanup.

use super::super::super::association::{
    DirectAssociatedReadmissionCleanupOutcome, DirectReadmissionCleanupPendingAssociation,
    DirectReadmissionCleanupReceiptAssociation, DirectReadmissionCleanupRequiredAssociation,
};
use crate::domain_computation::{
    WorthQueryBoundConvergenceReport, WorthQueryConvergenceEpochCounters,
    WorthQueryManagedRunCleanupDisposition, WorthQueryReadmissionEvidence,
    WorthQueryRetainedConvergenceCandidateEvidence,
};

#[must_use = "direct convergence readmission cleanup must be finished"]
pub struct WorthQueryDirectConvergenceReadmissionCleanupRequired {
    association: DirectReadmissionCleanupRequiredAssociation,
}

#[must_use = "direct convergence readmission cleanup outcome retains closed or retry authority"]
pub enum WorthQueryDirectConvergenceReadmissionCleanupOutcome {
    Complete(WorthQueryDirectConvergenceReadmissionCleanupReceipt),
    Pending(WorthQueryDirectConvergenceReadmissionCleanupPending),
    RecoveryRequired(WorthQueryDirectConvergenceReadmissionCleanupReceipt),
}

#[must_use = "pending direct convergence readmission cleanup must be retried"]
pub struct WorthQueryDirectConvergenceReadmissionCleanupPending {
    association: DirectReadmissionCleanupPendingAssociation,
}

pub struct WorthQueryDirectConvergenceReadmissionCleanupReceipt {
    association: DirectReadmissionCleanupReceiptAssociation,
}

impl WorthQueryDirectConvergenceReadmissionCleanupRequired {
    pub(super) fn from_association(
        association: DirectReadmissionCleanupRequiredAssociation,
    ) -> Self {
        Self { association }
    }

    pub fn finish(self) -> WorthQueryDirectConvergenceReadmissionCleanupOutcome {
        admit_cleanup_outcome(self.association.finish())
    }
}

impl WorthQueryDirectConvergenceReadmissionCleanupOutcome {
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

impl WorthQueryDirectConvergenceReadmissionCleanupPending {
    pub fn retry(self) -> WorthQueryDirectConvergenceReadmissionCleanupOutcome {
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

impl WorthQueryDirectConvergenceReadmissionCleanupReceipt {
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
    outcome: DirectAssociatedReadmissionCleanupOutcome,
) -> WorthQueryDirectConvergenceReadmissionCleanupOutcome {
    match outcome {
        DirectAssociatedReadmissionCleanupOutcome::Complete(association) => {
            WorthQueryDirectConvergenceReadmissionCleanupOutcome::Complete(
                WorthQueryDirectConvergenceReadmissionCleanupReceipt { association },
            )
        }
        DirectAssociatedReadmissionCleanupOutcome::Pending(association) => {
            WorthQueryDirectConvergenceReadmissionCleanupOutcome::Pending(
                WorthQueryDirectConvergenceReadmissionCleanupPending { association },
            )
        }
        DirectAssociatedReadmissionCleanupOutcome::RecoveryRequired(association) => {
            WorthQueryDirectConvergenceReadmissionCleanupOutcome::RecoveryRequired(
                WorthQueryDirectConvergenceReadmissionCleanupReceipt { association },
            )
        }
    }
}
