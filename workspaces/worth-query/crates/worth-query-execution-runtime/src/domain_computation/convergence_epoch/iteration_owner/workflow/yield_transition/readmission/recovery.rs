//! Public workflow convergence recovery after readmission cannot reassemble ownership.

use super::super::WorthQueryYieldedWorkflowConvergenceIteration;
use super::cleanup::WorthQueryWorkflowConvergenceReadmissionCleanupRequired;
use crate::domain_computation::convergence_epoch::iteration_owner::workflow::association::{
    WorkflowAssociatedReadmissionRecovery, WorkflowAssociatedYieldReassemblyOutcome,
    WorkflowReadmissionTerminalRecoveryAssociation,
    WorkflowReadmissionYieldReassemblyRecoveryAssociation,
};
use crate::domain_computation::WorthQueryReadmissionEvidence;

#[must_use = "convergence readmission recovery must be resolved by authority posture"]
pub enum WorthQueryWorkflowConvergenceReadmissionRecoveryRequired {
    YieldReassembly(WorthQueryWorkflowConvergenceReadmissionYieldReassemblyRecovery),
    TerminalCleanup(WorthQueryWorkflowConvergenceReadmissionTerminalRecovery),
}

#[must_use = "yield reassembly recovery must be retried or enter cleanup"]
pub struct WorthQueryWorkflowConvergenceReadmissionYieldReassemblyRecovery {
    association: WorkflowReadmissionYieldReassemblyRecoveryAssociation,
}

#[must_use = "terminal readmission recovery must enter cleanup"]
pub struct WorthQueryWorkflowConvergenceReadmissionTerminalRecovery {
    association: WorkflowReadmissionTerminalRecoveryAssociation,
}

#[must_use = "yield reassembly outcomes retain yielded or recovery authority"]
pub enum WorthQueryWorkflowConvergenceYieldReassemblyOutcome {
    Yielded(WorthQueryWorkflowConvergenceYieldReassembled),
    RecoveryRequired(WorthQueryWorkflowConvergenceReadmissionYieldReassemblyRecovery),
}

#[must_use = "reassembled convergence yield authority must be consumed"]
pub struct WorthQueryWorkflowConvergenceYieldReassembled {
    yielded: WorthQueryYieldedWorkflowConvergenceIteration,
    evidence: WorthQueryReadmissionEvidence,
}

impl WorthQueryWorkflowConvergenceReadmissionYieldReassemblyRecovery {
    pub fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.association.readmission_evidence()
    }

    pub fn retry_to_yielded(self) -> WorthQueryWorkflowConvergenceYieldReassemblyOutcome {
        match self.association.retry_to_yielded() {
            WorkflowAssociatedYieldReassemblyOutcome::Yielded {
                association,
                evidence,
            } => WorthQueryWorkflowConvergenceYieldReassemblyOutcome::Yielded(
                WorthQueryWorkflowConvergenceYieldReassembled {
                    yielded: WorthQueryYieldedWorkflowConvergenceIteration { association },
                    evidence,
                },
            ),
            WorkflowAssociatedYieldReassemblyOutcome::RecoveryRequired(association) => {
                WorthQueryWorkflowConvergenceYieldReassemblyOutcome::RecoveryRequired(Self {
                    association,
                })
            }
        }
    }

    pub fn into_cleanup(self) -> WorthQueryWorkflowConvergenceReadmissionCleanupRequired {
        WorthQueryWorkflowConvergenceReadmissionCleanupRequired::from_association(
            self.association.begin_readmission_cleanup(),
        )
    }
}

impl WorthQueryWorkflowConvergenceReadmissionTerminalRecovery {
    pub fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.association.readmission_evidence()
    }

    pub fn into_cleanup(self) -> WorthQueryWorkflowConvergenceReadmissionCleanupRequired {
        WorthQueryWorkflowConvergenceReadmissionCleanupRequired::from_association(
            self.association.begin_readmission_cleanup(),
        )
    }
}

impl WorthQueryWorkflowConvergenceYieldReassembled {
    pub const fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.evidence
    }

    pub fn into_yielded(self) -> WorthQueryYieldedWorkflowConvergenceIteration {
        self.yielded
    }
}

pub(super) fn admit_associated_recovery(
    recovery: WorkflowAssociatedReadmissionRecovery,
) -> WorthQueryWorkflowConvergenceReadmissionRecoveryRequired {
    match recovery {
        WorkflowAssociatedReadmissionRecovery::YieldReassembly(association) => {
            WorthQueryWorkflowConvergenceReadmissionRecoveryRequired::YieldReassembly(
                WorthQueryWorkflowConvergenceReadmissionYieldReassemblyRecovery { association },
            )
        }
        WorkflowAssociatedReadmissionRecovery::TerminalCleanup(association) => {
            WorthQueryWorkflowConvergenceReadmissionRecoveryRequired::TerminalCleanup(
                WorthQueryWorkflowConvergenceReadmissionTerminalRecovery { association },
            )
        }
    }
}
