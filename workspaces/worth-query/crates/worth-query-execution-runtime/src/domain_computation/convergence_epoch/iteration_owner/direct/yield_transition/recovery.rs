use super::super::super::super::{
    WorthQueryBoundConvergenceReport, WorthQueryConvergenceEpochCounters,
    WorthQueryRetainedConvergenceCandidateEvidence,
};
use super::super::association::{
    DirectAssociatedYieldRecovery, DirectYieldRecoveryCleanupReceiptAssociation,
    DirectYieldRunningRecoveryAssociation, DirectYieldTerminalCleanupAssociation,
};
use super::super::WorthQueryPausedDirectConvergenceIteration;

#[must_use = "yield recovery authority must be resumed or cleaned up"]
pub enum WorthQueryDirectConvergenceYieldRecoveryRequired {
    RunningAttempt(WorthQueryDirectConvergenceYieldRunningRecovery),
    TerminalCleanup(WorthQueryDirectConvergenceYieldTerminalCleanupRequired),
}

#[must_use = "running yield recovery must be resumed"]
pub struct WorthQueryDirectConvergenceYieldRunningRecovery {
    association: DirectYieldRunningRecoveryAssociation,
}

#[must_use = "terminal yield recovery must release its retained resources"]
pub struct WorthQueryDirectConvergenceYieldTerminalCleanupRequired {
    association: DirectYieldTerminalCleanupAssociation,
}

pub struct WorthQueryDirectConvergenceYieldRecoveryCleanupReceipt {
    association: DirectYieldRecoveryCleanupReceiptAssociation,
}

impl WorthQueryDirectConvergenceYieldRunningRecovery {
    pub fn resume(
        self,
    ) -> Result<
        WorthQueryPausedDirectConvergenceIteration,
        WorthQueryDirectConvergenceYieldTerminalCleanupRequired,
    > {
        self.association
            .resume()
            .map(|association| WorthQueryPausedDirectConvergenceIteration { association })
            .map_err(
                |association| WorthQueryDirectConvergenceYieldTerminalCleanupRequired {
                    association,
                },
            )
    }
}

impl WorthQueryDirectConvergenceYieldTerminalCleanupRequired {
    pub fn finish(
        self,
    ) -> Result<
        WorthQueryDirectConvergenceYieldRecoveryCleanupReceipt,
        WorthQueryDirectConvergenceYieldRunningRecovery,
    > {
        self.association
            .finish()
            .map(
                |association| WorthQueryDirectConvergenceYieldRecoveryCleanupReceipt {
                    association,
                },
            )
            .map_err(|association| WorthQueryDirectConvergenceYieldRunningRecovery { association })
    }
}

impl WorthQueryDirectConvergenceYieldRecoveryCleanupReceipt {
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
    recovery: DirectAssociatedYieldRecovery,
) -> WorthQueryDirectConvergenceYieldRecoveryRequired {
    match recovery {
        DirectAssociatedYieldRecovery::RunningAttempt(association) => {
            WorthQueryDirectConvergenceYieldRecoveryRequired::RunningAttempt(
                WorthQueryDirectConvergenceYieldRunningRecovery { association },
            )
        }
        DirectAssociatedYieldRecovery::TerminalCleanup(association) => {
            WorthQueryDirectConvergenceYieldRecoveryRequired::TerminalCleanup(
                WorthQueryDirectConvergenceYieldTerminalCleanupRequired { association },
            )
        }
    }
}
