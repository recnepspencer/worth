//! Workflow yield transformations that preserve one associated owner.

use super::{WorkflowIterationAssociation, WorthQueryConvergenceEpochCore};
use crate::domain_computation::{
    WorthQueryPausedWorkflowGraphExecution, WorthQueryWorkflowYieldDenied,
    WorthQueryWorkflowYieldOutcome, WorthQueryWorkflowYieldRecoveryRequired,
    WorthQueryYieldedWorkflowRun,
};

mod recovery;

pub use recovery::{
    WorkflowYieldRecoveryCleanupLifecycleEvent, WorkflowYieldRecoveryCleanupLifecycleEventKind,
};

pub(in super::super) enum WorkflowAssociatedYieldOutcome {
    Yielded(WorkflowIterationAssociation<WorthQueryYieldedWorkflowRun>),
    Denied(WorkflowIterationAssociation<WorthQueryWorkflowYieldDenied>),
    RecoveryRequired(WorkflowAssociatedYieldRecovery),
}

pub(in super::super) enum WorkflowAssociatedYieldRecovery {
    RunningAttempt(WorkflowYieldRunningRecoveryAssociation),
    TerminalCleanup(WorkflowYieldTerminalCleanupAssociation),
}

pub(in super::super) struct WorkflowYieldRunningRecoveryAssociation {
    association: WorkflowIterationAssociation<WorthQueryWorkflowYieldRecoveryRequired>,
}

pub(in super::super) struct WorkflowYieldTerminalCleanupAssociation {
    association: WorkflowIterationAssociation<WorthQueryWorkflowYieldRecoveryRequired>,
}

pub(in super::super) struct WorkflowYieldRecoveryCleanupPendingAssociation {
    association: WorkflowIterationAssociation<
        crate::domain_computation::WorthQueryWorkflowYieldRecoveryReleasePending,
    >,
}

pub(in super::super) struct WorkflowYieldRecoveryCleanupReceiptAssociation {
    core: WorthQueryConvergenceEpochCore,
}

pub(in super::super) enum WorkflowAssociatedYieldRecoveryCleanupOutcome {
    Complete(WorkflowYieldRecoveryCleanupReceiptAssociation),
    Pending(WorkflowYieldRecoveryCleanupPendingAssociation),
    RecoveryRequired(WorkflowYieldRecoveryCleanupReceiptAssociation),
}

impl WorkflowIterationAssociation<WorthQueryPausedWorkflowGraphExecution> {
    pub(in super::super) fn yield_iteration(self) -> WorkflowAssociatedYieldOutcome {
        let Self {
            mut core,
            graph,
            provider,
            stage_identity,
            managed,
        } = self;
        match managed.yield_run() {
            WorthQueryWorkflowYieldOutcome::Yielded(managed) => {
                core.record_lifecycle_event(WorkflowYieldedLifecycleEvent::new());
                WorkflowAssociatedYieldOutcome::Yielded(WorkflowIterationAssociation {
                    core,
                    graph,
                    provider,
                    stage_identity,
                    managed,
                })
            }
            WorthQueryWorkflowYieldOutcome::Denied(managed) => {
                WorkflowAssociatedYieldOutcome::Denied(WorkflowIterationAssociation {
                    core,
                    graph,
                    provider,
                    stage_identity,
                    managed,
                })
            }
            WorthQueryWorkflowYieldOutcome::RecoveryRequired(managed) => {
                let association = WorkflowIterationAssociation {
                    core,
                    graph,
                    provider,
                    stage_identity,
                    managed,
                };
                WorkflowAssociatedYieldOutcome::RecoveryRequired(
                    WorkflowAssociatedYieldRecovery::classify(association),
                )
            }
        }
    }
}

pub struct WorkflowYieldedLifecycleEvent {
    _permit: (),
}

impl WorkflowYieldedLifecycleEvent {
    fn new() -> Self {
        Self { _permit: () }
    }
}

impl WorkflowIterationAssociation<WorthQueryWorkflowYieldDenied> {
    pub(in super::super) fn retry(
        self,
    ) -> WorkflowIterationAssociation<WorthQueryPausedWorkflowGraphExecution> {
        let Self {
            core,
            graph,
            provider,
            stage_identity,
            managed,
        } = self;
        WorkflowIterationAssociation {
            core,
            graph,
            provider,
            stage_identity,
            managed: managed.into_paused(),
        }
    }
}

impl WorkflowIterationAssociation<WorthQueryYieldedWorkflowRun> {
    pub(in super::super) fn epoch_identity(&self) -> &str {
        self.core.identity()
    }

    pub(in super::super) fn logical_run_identity(&self) -> &str {
        self.core.logical_run_identity()
    }

    pub(in super::super) fn graph_authority_identity(&self) -> &str {
        self.graph.authority_identity()
    }
}
