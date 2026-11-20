use super::super::WorthQueryConvergenceEpochCore;
use super::{
    WorkflowAssociatedYieldRecovery, WorkflowAssociatedYieldRecoveryCleanupOutcome,
    WorkflowIterationAssociation, WorkflowYieldRecoveryCleanupPendingAssociation,
    WorkflowYieldRecoveryCleanupReceiptAssociation, WorkflowYieldRunningRecoveryAssociation,
    WorkflowYieldTerminalCleanupAssociation,
};
use crate::domain_computation::{
    WorthQueryPausedWorkflowGraphExecution, WorthQueryWorkflowYieldRecoveryReleaseOutcome,
    WorthQueryWorkflowYieldRecoveryRequired,
};

impl WorkflowAssociatedYieldRecovery {
    pub(super) fn classify(
        association: WorkflowIterationAssociation<WorthQueryWorkflowYieldRecoveryRequired>,
    ) -> Self {
        if association.managed.running_attempt_recoverable() {
            Self::RunningAttempt(WorkflowYieldRunningRecoveryAssociation { association })
        } else {
            Self::TerminalCleanup(WorkflowYieldTerminalCleanupAssociation { association })
        }
    }
}

impl WorkflowYieldRunningRecoveryAssociation {
    pub(in super::super::super) fn resume(
        self,
    ) -> Result<
        WorkflowIterationAssociation<WorthQueryPausedWorkflowGraphExecution>,
        WorkflowYieldTerminalCleanupAssociation,
    > {
        let WorkflowIterationAssociation {
            core,
            graph,
            provider,
            stage_identity,
            managed,
        } = self.association;
        match managed.into_paused() {
            Ok(managed) => Ok(WorkflowIterationAssociation {
                core,
                graph,
                provider,
                stage_identity,
                managed,
            }),
            Err(managed) => Err(WorkflowYieldTerminalCleanupAssociation {
                association: WorkflowIterationAssociation {
                    core,
                    graph,
                    provider,
                    stage_identity,
                    managed,
                },
            }),
        }
    }
}

impl WorkflowYieldTerminalCleanupAssociation {
    pub(in super::super::super) fn finish(
        self,
    ) -> Result<
        WorkflowAssociatedYieldRecoveryCleanupOutcome,
        WorkflowYieldRunningRecoveryAssociation,
    > {
        let WorkflowIterationAssociation {
            mut core,
            graph,
            provider,
            stage_identity,
            managed,
        } = self.association;
        core.record_lifecycle_event(WorkflowYieldRecoveryCleanupLifecycleEvent::attempted());
        match managed.release_terminalized() {
            Ok(managed) => Ok(map_release(WorkflowIterationAssociation {
                core,
                graph,
                provider,
                stage_identity,
                managed,
            })),
            Err(managed) => Err(WorkflowYieldRunningRecoveryAssociation {
                association: WorkflowIterationAssociation {
                    core,
                    graph,
                    provider,
                    stage_identity,
                    managed,
                },
            }),
        }
    }
}

impl WorkflowYieldRecoveryCleanupPendingAssociation {
    pub(in super::super::super) fn retry(
        self,
    ) -> Result<
        WorkflowAssociatedYieldRecoveryCleanupOutcome,
        WorkflowYieldRunningRecoveryAssociation,
    > {
        let WorkflowIterationAssociation {
            mut core,
            graph,
            provider,
            stage_identity,
            managed,
        } = self.association;
        core.record_lifecycle_event(WorkflowYieldRecoveryCleanupLifecycleEvent::attempted());
        match managed.retry() {
            Ok(managed) => Ok(map_release(WorkflowIterationAssociation {
                core,
                graph,
                provider,
                stage_identity,
                managed,
            })),
            Err(managed) => Err(WorkflowYieldRunningRecoveryAssociation {
                association: WorkflowIterationAssociation {
                    core,
                    graph,
                    provider,
                    stage_identity,
                    managed,
                },
            }),
        }
    }

    pub(in super::super::super) fn core(&self) -> &WorthQueryConvergenceEpochCore {
        &self.association.core
    }
}

impl WorkflowYieldRecoveryCleanupReceiptAssociation {
    pub(in super::super::super) fn core(&self) -> &WorthQueryConvergenceEpochCore {
        &self.core
    }
}

fn map_release(
    association: WorkflowIterationAssociation<WorthQueryWorkflowYieldRecoveryReleaseOutcome>,
) -> WorkflowAssociatedYieldRecoveryCleanupOutcome {
    let WorkflowIterationAssociation {
        mut core,
        graph,
        provider,
        stage_identity,
        managed,
    } = association;
    match managed {
        WorthQueryWorkflowYieldRecoveryReleaseOutcome::Complete(_closed) => {
            core.record_lifecycle_event(WorkflowYieldRecoveryCleanupLifecycleEvent::completed());
            WorkflowAssociatedYieldRecoveryCleanupOutcome::Complete(
                WorkflowYieldRecoveryCleanupReceiptAssociation { core },
            )
        }
        WorthQueryWorkflowYieldRecoveryReleaseOutcome::Pending(managed) => {
            WorkflowAssociatedYieldRecoveryCleanupOutcome::Pending(
                WorkflowYieldRecoveryCleanupPendingAssociation {
                    association: WorkflowIterationAssociation {
                        core,
                        graph,
                        provider,
                        stage_identity,
                        managed,
                    },
                },
            )
        }
        WorthQueryWorkflowYieldRecoveryReleaseOutcome::RecoveryRequired(_closed) => {
            core.record_lifecycle_event(WorkflowYieldRecoveryCleanupLifecycleEvent::completed());
            WorkflowAssociatedYieldRecoveryCleanupOutcome::RecoveryRequired(
                WorkflowYieldRecoveryCleanupReceiptAssociation { core },
            )
        }
    }
}

pub struct WorkflowYieldRecoveryCleanupLifecycleEvent
{
    kind: WorkflowYieldRecoveryCleanupLifecycleEventKind,
}

pub enum WorkflowYieldRecoveryCleanupLifecycleEventKind
{
    Attempted,
    Completed,
}

impl WorkflowYieldRecoveryCleanupLifecycleEvent {
    fn attempted() -> Self {
        Self {
            kind: WorkflowYieldRecoveryCleanupLifecycleEventKind::Attempted,
        }
    }

    fn completed() -> Self {
        Self {
            kind: WorkflowYieldRecoveryCleanupLifecycleEventKind::Completed,
        }
    }

    pub fn into_kind(
        self,
    ) -> WorkflowYieldRecoveryCleanupLifecycleEventKind {
        self.kind
    }
}
