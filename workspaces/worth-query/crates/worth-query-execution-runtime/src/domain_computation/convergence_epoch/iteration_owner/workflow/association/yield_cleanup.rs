//! Workflow yielded-run cleanup under one sealed convergence association.

use super::{WorkflowIterationAssociation, WorthQueryConvergenceEpochCore};
use crate::domain_computation::{
    WorthQueryWorkflowYieldCleanupOutcome, WorthQueryWorkflowYieldCleanupPending,
    WorthQueryYieldedWorkflowRun,
};

pub(in super::super) enum WorkflowAssociatedYieldCleanupOutcome {
    Complete(WorkflowYieldCleanupReceiptAssociation),
    Pending(WorkflowYieldCleanupPendingAssociation),
    RecoveryRequired(WorkflowYieldCleanupReceiptAssociation),
}

pub(in super::super) struct WorkflowYieldCleanupPendingAssociation {
    core: WorthQueryConvergenceEpochCore,
    managed: WorthQueryWorkflowYieldCleanupPending,
}

pub(in super::super) struct WorkflowYieldCleanupReceiptAssociation {
    core: WorthQueryConvergenceEpochCore,
}

impl WorkflowIterationAssociation<WorthQueryYieldedWorkflowRun> {
    pub(in super::super) fn cleanup(self) -> WorkflowAssociatedYieldCleanupOutcome {
        let Self {
            mut core,
            graph: _,
            provider: _,
            stage_identity: _,
            managed,
        } = self;
        core.record_lifecycle_event(WorkflowYieldCleanupLifecycleEvent::attempted());
        admit_cleanup_outcome(core, managed.cleanup())
    }
}

impl WorkflowYieldCleanupPendingAssociation {
    pub(in super::super) fn core(&self) -> &WorthQueryConvergenceEpochCore {
        &self.core
    }

    pub(in super::super) fn retry(mut self) -> WorkflowAssociatedYieldCleanupOutcome {
        self.core
            .record_lifecycle_event(WorkflowYieldCleanupLifecycleEvent::attempted());
        admit_cleanup_outcome(self.core, self.managed.retry())
    }
}

impl WorkflowYieldCleanupReceiptAssociation {
    pub(in super::super) fn core(&self) -> &WorthQueryConvergenceEpochCore {
        &self.core
    }
}

fn admit_cleanup_outcome(
    mut core: WorthQueryConvergenceEpochCore,
    outcome: WorthQueryWorkflowYieldCleanupOutcome,
) -> WorkflowAssociatedYieldCleanupOutcome {
    match outcome {
        WorthQueryWorkflowYieldCleanupOutcome::Complete(_closed_receipt) => {
            core.record_lifecycle_event(WorkflowYieldCleanupLifecycleEvent::completed());
            WorkflowAssociatedYieldCleanupOutcome::Complete(
                WorkflowYieldCleanupReceiptAssociation { core },
            )
        }
        WorthQueryWorkflowYieldCleanupOutcome::Pending(managed) => {
            WorkflowAssociatedYieldCleanupOutcome::Pending(WorkflowYieldCleanupPendingAssociation {
                core,
                managed,
            })
        }
        WorthQueryWorkflowYieldCleanupOutcome::RecoveryRequired(_closed_receipt) => {
            core.record_lifecycle_event(WorkflowYieldCleanupLifecycleEvent::completed());
            WorkflowAssociatedYieldCleanupOutcome::RecoveryRequired(
                WorkflowYieldCleanupReceiptAssociation { core },
            )
        }
    }
}

pub struct WorkflowYieldCleanupLifecycleEvent {
    kind: WorkflowYieldCleanupLifecycleEventKind,
}

pub enum WorkflowYieldCleanupLifecycleEventKind {
    Attempted,
    Completed,
}

impl WorkflowYieldCleanupLifecycleEvent {
    fn attempted() -> Self {
        Self {
            kind: WorkflowYieldCleanupLifecycleEventKind::Attempted,
        }
    }

    fn completed() -> Self {
        Self {
            kind: WorkflowYieldCleanupLifecycleEventKind::Completed,
        }
    }

    pub fn into_kind(
        self,
    ) -> WorkflowYieldCleanupLifecycleEventKind {
        self.kind
    }
}
