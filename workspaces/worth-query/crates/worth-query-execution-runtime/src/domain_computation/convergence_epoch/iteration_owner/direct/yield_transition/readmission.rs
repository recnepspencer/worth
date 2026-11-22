//! Direct-lane readmission and yielded-owner reassembly transitions.

use super::super::{
    association::{DirectAssociatedReadmissionOutcome, DirectIterationAssociation},
    WorthQueryStartedDirectConvergenceIteration,
};
use super::WorthQueryYieldedDirectConvergenceIteration;
use crate::domain_computation::{WorthQueryDirectReadmissionDenied, WorthQueryReadmissionEvidence};

#[path = "readmission/cleanup.rs"]
mod cleanup;
#[path = "readmission/recovery.rs"]
mod recovery;

pub use cleanup::{
    WorthQueryDirectConvergenceReadmissionCleanupOutcome,
    WorthQueryDirectConvergenceReadmissionCleanupPending,
    WorthQueryDirectConvergenceReadmissionCleanupReceipt,
    WorthQueryDirectConvergenceReadmissionCleanupRequired,
};
pub use recovery::{
    WorthQueryDirectConvergenceReadmissionRecoveryRequired,
    WorthQueryDirectConvergenceReadmissionTerminalRecovery,
    WorthQueryDirectConvergenceReadmissionYieldReassemblyRecovery,
    WorthQueryDirectConvergenceYieldReassembled, WorthQueryDirectConvergenceYieldReassemblyOutcome,
};

#[must_use = "readmitted convergence iteration must continue through its started authority"]
pub struct WorthQueryReadmittedDirectConvergenceIteration {
    started: WorthQueryStartedDirectConvergenceIteration,
    evidence: WorthQueryReadmissionEvidence,
}

impl WorthQueryReadmittedDirectConvergenceIteration {
    pub const fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.evidence
    }

    pub fn into_started(self) -> WorthQueryStartedDirectConvergenceIteration {
        self.started
    }
}

#[must_use = "convergence readmission denial retains exact yielded iteration authority"]
pub struct WorthQueryDirectConvergenceReadmissionDenied {
    association: DirectIterationAssociation<WorthQueryDirectReadmissionDenied>,
}

impl WorthQueryDirectConvergenceReadmissionDenied {
    pub fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.association.readmission_evidence()
    }

    #[must_use = "readmission denial returns the exact yielded direct convergence iteration"]
    pub fn into_yielded(self) -> WorthQueryYieldedDirectConvergenceIteration {
        WorthQueryYieldedDirectConvergenceIteration {
            association: self.association.into_yielded(),
        }
    }
}

#[must_use = "convergence readmission outcomes retain started, yielded, or recovery authority"]
pub enum WorthQueryDirectConvergenceReadmissionOutcome {
    Readmitted(WorthQueryReadmittedDirectConvergenceIteration),
    Denied(WorthQueryDirectConvergenceReadmissionDenied),
    RecoveryRequired(WorthQueryDirectConvergenceReadmissionRecoveryRequired),
}

pub(super) fn admit_associated_readmission(
    outcome: DirectAssociatedReadmissionOutcome,
) -> WorthQueryDirectConvergenceReadmissionOutcome {
    match outcome {
        DirectAssociatedReadmissionOutcome::Readmitted {
            association,
            evidence,
        } => WorthQueryDirectConvergenceReadmissionOutcome::Readmitted(
            WorthQueryReadmittedDirectConvergenceIteration {
                started: WorthQueryStartedDirectConvergenceIteration { association },
                evidence,
            },
        ),
        DirectAssociatedReadmissionOutcome::Denied(association) => {
            WorthQueryDirectConvergenceReadmissionOutcome::Denied(
                WorthQueryDirectConvergenceReadmissionDenied { association },
            )
        }
        DirectAssociatedReadmissionOutcome::RecoveryRequired(recovery) => {
            WorthQueryDirectConvergenceReadmissionOutcome::RecoveryRequired(
                recovery::admit_associated_recovery(recovery),
            )
        }
    }
}
