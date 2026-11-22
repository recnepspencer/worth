//! Public direct convergence recovery after readmission cannot reassemble ownership.

use super::super::WorthQueryYieldedDirectConvergenceIteration;
use super::cleanup::WorthQueryDirectConvergenceReadmissionCleanupRequired;
use crate::domain_computation::convergence_epoch::iteration_owner::direct::association::{
    DirectAssociatedReadmissionRecovery, DirectAssociatedYieldReassemblyOutcome,
    DirectReadmissionTerminalRecoveryAssociation,
    DirectReadmissionYieldReassemblyRecoveryAssociation,
};
use crate::domain_computation::WorthQueryReadmissionEvidence;

#[must_use = "convergence readmission recovery must be resolved by authority posture"]
pub enum WorthQueryDirectConvergenceReadmissionRecoveryRequired {
    YieldReassembly(WorthQueryDirectConvergenceReadmissionYieldReassemblyRecovery),
    TerminalCleanup(WorthQueryDirectConvergenceReadmissionTerminalRecovery),
}

#[must_use = "yield reassembly recovery must be retried or enter cleanup"]
pub struct WorthQueryDirectConvergenceReadmissionYieldReassemblyRecovery {
    association: DirectReadmissionYieldReassemblyRecoveryAssociation,
}

#[must_use = "terminal readmission recovery must enter cleanup"]
pub struct WorthQueryDirectConvergenceReadmissionTerminalRecovery {
    association: DirectReadmissionTerminalRecoveryAssociation,
}

#[must_use = "yield reassembly outcomes retain yielded or recovery authority"]
pub enum WorthQueryDirectConvergenceYieldReassemblyOutcome {
    Yielded(WorthQueryDirectConvergenceYieldReassembled),
    RecoveryRequired(WorthQueryDirectConvergenceReadmissionYieldReassemblyRecovery),
}

#[must_use = "reassembled convergence yield authority must be consumed"]
pub struct WorthQueryDirectConvergenceYieldReassembled {
    yielded: WorthQueryYieldedDirectConvergenceIteration,
    evidence: WorthQueryReadmissionEvidence,
}

impl WorthQueryDirectConvergenceReadmissionYieldReassemblyRecovery {
    pub fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.association.readmission_evidence()
    }

    pub fn retry_to_yielded(self) -> WorthQueryDirectConvergenceYieldReassemblyOutcome {
        match self.association.retry_to_yielded() {
            DirectAssociatedYieldReassemblyOutcome::Yielded {
                association,
                evidence,
            } => WorthQueryDirectConvergenceYieldReassemblyOutcome::Yielded(
                WorthQueryDirectConvergenceYieldReassembled {
                    yielded: WorthQueryYieldedDirectConvergenceIteration { association },
                    evidence,
                },
            ),
            DirectAssociatedYieldReassemblyOutcome::RecoveryRequired(association) => {
                WorthQueryDirectConvergenceYieldReassemblyOutcome::RecoveryRequired(Self {
                    association,
                })
            }
        }
    }

    pub fn into_cleanup(self) -> WorthQueryDirectConvergenceReadmissionCleanupRequired {
        WorthQueryDirectConvergenceReadmissionCleanupRequired::from_association(
            self.association.begin_readmission_cleanup(),
        )
    }
}

impl WorthQueryDirectConvergenceReadmissionTerminalRecovery {
    pub fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.association.readmission_evidence()
    }

    pub fn into_cleanup(self) -> WorthQueryDirectConvergenceReadmissionCleanupRequired {
        WorthQueryDirectConvergenceReadmissionCleanupRequired::from_association(
            self.association.begin_readmission_cleanup(),
        )
    }
}

impl WorthQueryDirectConvergenceYieldReassembled {
    pub const fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.evidence
    }

    pub fn into_yielded(self) -> WorthQueryYieldedDirectConvergenceIteration {
        self.yielded
    }
}

pub(super) fn admit_associated_recovery(
    recovery: DirectAssociatedReadmissionRecovery,
) -> WorthQueryDirectConvergenceReadmissionRecoveryRequired {
    match recovery {
        DirectAssociatedReadmissionRecovery::YieldReassembly(association) => {
            WorthQueryDirectConvergenceReadmissionRecoveryRequired::YieldReassembly(
                WorthQueryDirectConvergenceReadmissionYieldReassemblyRecovery { association },
            )
        }
        DirectAssociatedReadmissionRecovery::TerminalCleanup(association) => {
            WorthQueryDirectConvergenceReadmissionRecoveryRequired::TerminalCleanup(
                WorthQueryDirectConvergenceReadmissionTerminalRecovery { association },
            )
        }
    }
}
