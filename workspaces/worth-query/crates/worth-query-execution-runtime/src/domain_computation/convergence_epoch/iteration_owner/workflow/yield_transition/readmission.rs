//! Workflow-lane readmission and yielded-owner reassembly transitions.

use super::super::{
    association::{WorkflowAssociatedReadmissionOutcome, WorkflowIterationAssociation},
    WorthQueryStartedWorkflowConvergenceIteration,
};
use super::WorthQueryYieldedWorkflowConvergenceIteration;
use crate::domain_computation::{
    WorthQueryReadmissionEvidence, WorthQueryWorkflowReadmissionDenied,
};

#[path = "readmission/cleanup.rs"]
mod cleanup;
#[path = "readmission/recovery.rs"]
mod recovery;

pub use cleanup::{
    WorthQueryWorkflowConvergenceReadmissionCleanupOutcome,
    WorthQueryWorkflowConvergenceReadmissionCleanupPending,
    WorthQueryWorkflowConvergenceReadmissionCleanupReceipt,
    WorthQueryWorkflowConvergenceReadmissionCleanupRequired,
};
pub use recovery::{
    WorthQueryWorkflowConvergenceReadmissionRecoveryRequired,
    WorthQueryWorkflowConvergenceReadmissionTerminalRecovery,
    WorthQueryWorkflowConvergenceReadmissionYieldReassemblyRecovery,
    WorthQueryWorkflowConvergenceYieldReassembled,
    WorthQueryWorkflowConvergenceYieldReassemblyOutcome,
};

#[must_use = "readmitted convergence iteration must continue through its started authority"]
pub struct WorthQueryReadmittedWorkflowConvergenceIteration {
    started: WorthQueryStartedWorkflowConvergenceIteration,
    evidence: WorthQueryReadmissionEvidence,
}

impl WorthQueryReadmittedWorkflowConvergenceIteration {
    pub const fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.evidence
    }

    pub fn into_started(self) -> WorthQueryStartedWorkflowConvergenceIteration {
        self.started
    }
}

#[must_use = "convergence readmission denial retains exact yielded iteration authority"]
pub struct WorthQueryWorkflowConvergenceReadmissionDenied {
    association: WorkflowIterationAssociation<WorthQueryWorkflowReadmissionDenied>,
}

impl WorthQueryWorkflowConvergenceReadmissionDenied {
    pub fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.association.readmission_evidence()
    }

    #[must_use = "readmission denial returns the exact yielded workflow convergence iteration"]
    pub fn into_yielded(self) -> WorthQueryYieldedWorkflowConvergenceIteration {
        WorthQueryYieldedWorkflowConvergenceIteration {
            association: self.association.into_yielded(),
        }
    }
}

#[must_use = "convergence readmission outcomes retain started, yielded, or recovery authority"]
pub enum WorthQueryWorkflowConvergenceReadmissionOutcome {
    Readmitted(WorthQueryReadmittedWorkflowConvergenceIteration),
    Denied(WorthQueryWorkflowConvergenceReadmissionDenied),
    RecoveryRequired(WorthQueryWorkflowConvergenceReadmissionRecoveryRequired),
}

pub(super) fn admit_associated_readmission(
    outcome: WorkflowAssociatedReadmissionOutcome,
) -> WorthQueryWorkflowConvergenceReadmissionOutcome {
    match outcome {
        WorkflowAssociatedReadmissionOutcome::Readmitted {
            association,
            evidence,
        } => WorthQueryWorkflowConvergenceReadmissionOutcome::Readmitted(
            WorthQueryReadmittedWorkflowConvergenceIteration {
                started: WorthQueryStartedWorkflowConvergenceIteration { association },
                evidence,
            },
        ),
        WorkflowAssociatedReadmissionOutcome::Denied(association) => {
            WorthQueryWorkflowConvergenceReadmissionOutcome::Denied(
                WorthQueryWorkflowConvergenceReadmissionDenied { association },
            )
        }
        WorkflowAssociatedReadmissionOutcome::RecoveryRequired(recovery) => {
            WorthQueryWorkflowConvergenceReadmissionOutcome::RecoveryRequired(
                recovery::admit_associated_recovery(recovery),
            )
        }
    }
}
