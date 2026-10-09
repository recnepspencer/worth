//! Workflow yield ownership, including subordinate readmission transitions.

use super::association::{WorkflowAssociatedYieldOutcome, WorkflowIterationAssociation};
use crate::domain_computation::{WorthQueryWorkflowYieldDenied, WorthQueryYieldedWorkflowRun};

mod cleanup;
mod readmission;
mod recovery;

pub use cleanup::{
    WorthQueryWorkflowConvergenceYieldCleanupOutcome,
    WorthQueryWorkflowConvergenceYieldCleanupPending,
    WorthQueryWorkflowConvergenceYieldCleanupReceipt,
};
pub use readmission::{
    WorthQueryReadmittedWorkflowConvergenceIteration,
    WorthQueryWorkflowConvergenceReadmissionCleanupOutcome,
    WorthQueryWorkflowConvergenceReadmissionCleanupPending,
    WorthQueryWorkflowConvergenceReadmissionCleanupReceipt,
    WorthQueryWorkflowConvergenceReadmissionCleanupRequired,
    WorthQueryWorkflowConvergenceReadmissionDenied,
    WorthQueryWorkflowConvergenceReadmissionOutcome,
    WorthQueryWorkflowConvergenceReadmissionRecoveryRequired,
    WorthQueryWorkflowConvergenceReadmissionTerminalRecovery,
    WorthQueryWorkflowConvergenceReadmissionYieldReassemblyRecovery,
    WorthQueryWorkflowConvergenceYieldReassembled,
    WorthQueryWorkflowConvergenceYieldReassemblyOutcome,
};
pub use recovery::{
    WorthQueryWorkflowConvergenceYieldRecoveryCleanupOutcome,
    WorthQueryWorkflowConvergenceYieldRecoveryCleanupPending,
    WorthQueryWorkflowConvergenceYieldRecoveryCleanupReceipt,
    WorthQueryWorkflowConvergenceYieldRecoveryRequired,
    WorthQueryWorkflowConvergenceYieldRunningRecovery,
    WorthQueryWorkflowConvergenceYieldTerminalCleanupRequired,
};

#[must_use = "workflow convergence yield outcome must be resolved"]
pub enum WorthQueryWorkflowConvergenceYieldOutcome {
    Yielded(WorthQueryYieldedWorkflowConvergenceIteration),
    Denied(WorthQueryDeniedWorkflowConvergenceYield),
    RecoveryRequired(WorthQueryWorkflowConvergenceYieldRecoveryRequired),
}

#[must_use = "workflow convergence yield denial retains paused iteration authority"]
pub struct WorthQueryDeniedWorkflowConvergenceYield {
    association: WorkflowIterationAssociation<WorthQueryWorkflowYieldDenied>,
}

impl WorthQueryDeniedWorkflowConvergenceYield {
    #[must_use = "retry returns the exact paused workflow convergence iteration"]
    pub fn retry(self) -> super::WorthQueryPausedWorkflowConvergenceIteration {
        super::WorthQueryPausedWorkflowConvergenceIteration {
            association: self.association.retry(),
        }
    }
}

pub struct WorthQueryYieldedWorkflowConvergenceIteration {
    association: WorkflowIterationAssociation<WorthQueryYieldedWorkflowRun>,
}

impl WorthQueryYieldedWorkflowConvergenceIteration {
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
    ) -> WorthQueryWorkflowConvergenceReadmissionOutcome {
        readmission::admit_associated_readmission(self.association.readmit_same_runtime(
            phase,
            query_runtime,
            bridge_runtime,
        ))
    }

    pub fn cleanup(self) -> WorthQueryWorkflowConvergenceYieldCleanupOutcome {
        cleanup::admit_associated_cleanup(self.association.cleanup())
    }
}

pub(super) fn admit_associated_yield(
    outcome: WorkflowAssociatedYieldOutcome,
) -> WorthQueryWorkflowConvergenceYieldOutcome {
    match outcome {
        WorkflowAssociatedYieldOutcome::Yielded(association) => {
            WorthQueryWorkflowConvergenceYieldOutcome::Yielded(
                WorthQueryYieldedWorkflowConvergenceIteration { association },
            )
        }
        WorkflowAssociatedYieldOutcome::Denied(association) => {
            WorthQueryWorkflowConvergenceYieldOutcome::Denied(
                WorthQueryDeniedWorkflowConvergenceYield { association },
            )
        }
        WorkflowAssociatedYieldOutcome::RecoveryRequired(recovery) => {
            WorthQueryWorkflowConvergenceYieldOutcome::RecoveryRequired(
                recovery::admit_associated_recovery(recovery),
            )
        }
    }
}
