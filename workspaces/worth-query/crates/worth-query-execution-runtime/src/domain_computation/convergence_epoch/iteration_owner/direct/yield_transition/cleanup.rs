//! Public direct convergence cleanup phases after a yielded iteration.

use super::super::super::super::{
    WorthQueryBoundConvergenceReport, WorthQueryConvergenceEpochCounters,
    WorthQueryRetainedConvergenceCandidateEvidence,
};
use super::super::association::{
    DirectAssociatedYieldCleanupOutcome, DirectYieldCleanupReceiptAssociation,
};

#[must_use = "convergence yield cleanup carries a closed release posture"]
pub enum WorthQueryDirectConvergenceYieldCleanupOutcome {
    Complete(WorthQueryDirectConvergenceYieldCleanupReceipt),
    RecoveryRequired(WorthQueryDirectConvergenceYieldCleanupReceipt),
}

pub struct WorthQueryDirectConvergenceYieldCleanupReceipt {
    association: DirectYieldCleanupReceiptAssociation,
}

impl WorthQueryDirectConvergenceYieldCleanupReceipt {
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
    outcome: DirectAssociatedYieldCleanupOutcome,
) -> WorthQueryDirectConvergenceYieldCleanupOutcome {
    match outcome {
        DirectAssociatedYieldCleanupOutcome::Complete(association) => {
            WorthQueryDirectConvergenceYieldCleanupOutcome::Complete(
                WorthQueryDirectConvergenceYieldCleanupReceipt { association },
            )
        }
        DirectAssociatedYieldCleanupOutcome::RecoveryRequired(association) => {
            WorthQueryDirectConvergenceYieldCleanupOutcome::RecoveryRequired(
                WorthQueryDirectConvergenceYieldCleanupReceipt { association },
            )
        }
    }
}
