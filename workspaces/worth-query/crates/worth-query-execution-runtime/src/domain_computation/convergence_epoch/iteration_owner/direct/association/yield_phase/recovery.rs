use super::super::WorthQueryConvergenceEpochCore;
use super::{
    DirectAssociatedYieldRecovery, DirectIterationAssociation,
    DirectYieldRecoveryCleanupReceiptAssociation, DirectYieldRunningRecoveryAssociation,
    DirectYieldTerminalCleanupAssociation,
};
use crate::domain_computation::{
    WorthQueryDirectYieldRecoveryRequired, WorthQueryPausedDirectGraphExecution,
};

impl DirectAssociatedYieldRecovery {
    pub(super) fn classify(
        association: DirectIterationAssociation<WorthQueryDirectYieldRecoveryRequired>,
    ) -> Self {
        if association.managed.running_attempt_recoverable() {
            Self::RunningAttempt(DirectYieldRunningRecoveryAssociation { association })
        } else {
            Self::TerminalCleanup(DirectYieldTerminalCleanupAssociation { association })
        }
    }
}

impl DirectYieldRunningRecoveryAssociation {
    pub(in super::super::super) fn resume(
        self,
    ) -> Result<
        DirectIterationAssociation<WorthQueryPausedDirectGraphExecution>,
        DirectYieldTerminalCleanupAssociation,
    > {
        let DirectIterationAssociation {
            core,
            graph,
            provider,
            managed,
        } = self.association;
        match managed.into_paused() {
            Ok(managed) => Ok(DirectIterationAssociation {
                core,
                graph,
                provider,
                managed,
            }),
            Err(managed) => Err(DirectYieldTerminalCleanupAssociation {
                association: DirectIterationAssociation {
                    core,
                    graph,
                    provider,
                    managed,
                },
            }),
        }
    }
}

impl DirectYieldTerminalCleanupAssociation {
    pub(in super::super::super) fn finish(
        self,
    ) -> Result<DirectYieldRecoveryCleanupReceiptAssociation, DirectYieldRunningRecoveryAssociation>
    {
        let DirectIterationAssociation {
            mut core,
            graph,
            provider,
            managed,
        } = self.association;
        core.record_lifecycle_event(DirectYieldRecoveryCleanupLifecycleEvent::attempted());
        match managed.cleanup_terminalized() {
            Ok(_closed) => {
                core.record_lifecycle_event(DirectYieldRecoveryCleanupLifecycleEvent::completed());
                Ok(DirectYieldRecoveryCleanupReceiptAssociation { core })
            }
            Err(managed) => Err(DirectYieldRunningRecoveryAssociation {
                association: DirectIterationAssociation {
                    core,
                    graph,
                    provider,
                    managed,
                },
            }),
        }
    }
}

pub struct DirectYieldRecoveryCleanupLifecycleEvent {
    kind: DirectYieldRecoveryCleanupLifecycleEventKind,
}

pub enum DirectYieldRecoveryCleanupLifecycleEventKind {
    Attempted,
    Completed,
}

impl DirectYieldRecoveryCleanupLifecycleEvent {
    fn attempted() -> Self {
        Self {
            kind: DirectYieldRecoveryCleanupLifecycleEventKind::Attempted,
        }
    }

    fn completed() -> Self {
        Self {
            kind: DirectYieldRecoveryCleanupLifecycleEventKind::Completed,
        }
    }

    pub fn into_kind(self) -> DirectYieldRecoveryCleanupLifecycleEventKind {
        self.kind
    }
}

impl DirectYieldRecoveryCleanupReceiptAssociation {
    pub(in super::super::super) fn core(&self) -> &WorthQueryConvergenceEpochCore {
        &self.core
    }
}
