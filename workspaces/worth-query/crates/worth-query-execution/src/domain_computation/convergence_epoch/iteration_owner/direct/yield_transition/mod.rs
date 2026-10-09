//! Direct yield ownership, including subordinate readmission transitions.

use super::association::{DirectAssociatedYieldOutcome, DirectIterationAssociation};
use crate::domain_computation::{WorthQueryDirectYieldDenied, WorthQueryYieldedDirectRun};

mod cleanup;
mod readmission;
mod recovery;

pub use cleanup::{
    WorthQueryDirectConvergenceYieldCleanupOutcome, WorthQueryDirectConvergenceYieldCleanupReceipt,
};
pub use readmission::{
    WorthQueryDirectConvergenceReadmissionCleanupOutcome,
    WorthQueryDirectConvergenceReadmissionCleanupPending,
    WorthQueryDirectConvergenceReadmissionCleanupReceipt,
    WorthQueryDirectConvergenceReadmissionCleanupRequired,
    WorthQueryDirectConvergenceReadmissionDenied, WorthQueryDirectConvergenceReadmissionOutcome,
    WorthQueryDirectConvergenceReadmissionRecoveryRequired,
    WorthQueryDirectConvergenceReadmissionTerminalRecovery,
    WorthQueryDirectConvergenceReadmissionYieldReassemblyRecovery,
    WorthQueryDirectConvergenceYieldReassembled, WorthQueryDirectConvergenceYieldReassemblyOutcome,
    WorthQueryReadmittedDirectConvergenceIteration,
};
pub use recovery::{
    WorthQueryDirectConvergenceYieldRecoveryCleanupReceipt,
    WorthQueryDirectConvergenceYieldRecoveryRequired,
    WorthQueryDirectConvergenceYieldRunningRecovery,
    WorthQueryDirectConvergenceYieldTerminalCleanupRequired,
};

#[must_use = "direct convergence yield outcome must be resolved"]
pub enum WorthQueryDirectConvergenceYieldOutcome {
    Yielded(WorthQueryYieldedDirectConvergenceIteration),
    Denied(WorthQueryDeniedDirectConvergenceYield),
    RecoveryRequired(WorthQueryDirectConvergenceYieldRecoveryRequired),
}

#[must_use = "direct convergence yield denial retains paused iteration authority"]
pub struct WorthQueryDeniedDirectConvergenceYield {
    association: DirectIterationAssociation<WorthQueryDirectYieldDenied>,
}

impl WorthQueryDeniedDirectConvergenceYield {
    #[must_use = "retry returns the exact paused direct convergence iteration"]
    pub fn retry(self) -> super::WorthQueryPausedDirectConvergenceIteration {
        super::WorthQueryPausedDirectConvergenceIteration {
            association: self.association.retry(),
        }
    }
}

pub struct WorthQueryYieldedDirectConvergenceIteration {
    association: DirectIterationAssociation<WorthQueryYieldedDirectRun>,
}

impl WorthQueryYieldedDirectConvergenceIteration {
    pub fn epoch_identity(&self) -> &str {
        self.association.epoch_identity()
    }

    pub fn logical_run_identity(&self) -> &str {
        self.association.logical_run_identity()
    }

    pub fn graph_authority_identity(&self) -> &str {
        self.association.graph_authority_identity()
    }

    pub fn readmit_same_runtime(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

        query_runtime: &crate::domain_computation::WorthQueryExecutionRuntime,
        bridge_runtime: &worth_runtime_bridge::facade::RuntimeBridge,
    ) -> WorthQueryDirectConvergenceReadmissionOutcome {
        readmission::admit_associated_readmission(self.association.readmit_same_runtime(
            phase,
            query_runtime,
            bridge_runtime,
        ))
    }

    pub fn cleanup(self) -> WorthQueryDirectConvergenceYieldCleanupOutcome {
        cleanup::admit_associated_cleanup(self.association.cleanup())
    }
}

pub(super) fn admit_associated_yield(
    outcome: DirectAssociatedYieldOutcome,
) -> WorthQueryDirectConvergenceYieldOutcome {
    match outcome {
        DirectAssociatedYieldOutcome::Yielded(association) => {
            WorthQueryDirectConvergenceYieldOutcome::Yielded(
                WorthQueryYieldedDirectConvergenceIteration { association },
            )
        }
        DirectAssociatedYieldOutcome::Denied(association) => {
            WorthQueryDirectConvergenceYieldOutcome::Denied(
                WorthQueryDeniedDirectConvergenceYield { association },
            )
        }
        DirectAssociatedYieldOutcome::RecoveryRequired(recovery) => {
            WorthQueryDirectConvergenceYieldOutcome::RecoveryRequired(
                recovery::admit_associated_recovery(recovery),
            )
        }
    }
}
