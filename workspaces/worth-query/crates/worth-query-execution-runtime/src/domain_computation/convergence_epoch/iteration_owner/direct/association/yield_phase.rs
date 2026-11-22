//! Direct yield transformations that preserve one associated owner.

use super::{DirectIterationAssociation, WorthQueryConvergenceEpochCore};
use crate::domain_computation::{
    WorthQueryDirectYieldDenied, WorthQueryDirectYieldOutcome,
    WorthQueryDirectYieldRecoveryRequired, WorthQueryPausedDirectGraphExecution,
    WorthQueryYieldedDirectRun,
};

mod recovery;

pub use recovery::{
    DirectYieldRecoveryCleanupLifecycleEvent, DirectYieldRecoveryCleanupLifecycleEventKind,
};

pub(in super::super) enum DirectAssociatedYieldOutcome {
    Yielded(DirectIterationAssociation<WorthQueryYieldedDirectRun>),
    Denied(DirectIterationAssociation<WorthQueryDirectYieldDenied>),
    RecoveryRequired(DirectAssociatedYieldRecovery),
}

pub(in super::super) enum DirectAssociatedYieldRecovery {
    RunningAttempt(DirectYieldRunningRecoveryAssociation),
    TerminalCleanup(DirectYieldTerminalCleanupAssociation),
}

pub(in super::super) struct DirectYieldRunningRecoveryAssociation {
    association: DirectIterationAssociation<WorthQueryDirectYieldRecoveryRequired>,
}

pub(in super::super) struct DirectYieldTerminalCleanupAssociation {
    association: DirectIterationAssociation<WorthQueryDirectYieldRecoveryRequired>,
}

pub(in super::super) struct DirectYieldRecoveryCleanupReceiptAssociation {
    core: WorthQueryConvergenceEpochCore,
}

impl DirectIterationAssociation<WorthQueryPausedDirectGraphExecution> {
    pub(in super::super) fn yield_iteration(self) -> DirectAssociatedYieldOutcome {
        let Self {
            mut core,
            graph,
            provider,
            managed,
        } = self;
        match managed.yield_run() {
            WorthQueryDirectYieldOutcome::Yielded(managed) => {
                core.record_lifecycle_event(DirectYieldedLifecycleEvent::new());
                DirectAssociatedYieldOutcome::Yielded(DirectIterationAssociation {
                    core,
                    graph,
                    provider,
                    managed,
                })
            }
            WorthQueryDirectYieldOutcome::Denied(managed) => {
                DirectAssociatedYieldOutcome::Denied(DirectIterationAssociation {
                    core,
                    graph,
                    provider,
                    managed,
                })
            }
            WorthQueryDirectYieldOutcome::RecoveryRequired(managed) => {
                let association = DirectIterationAssociation {
                    core,
                    graph,
                    provider,
                    managed,
                };
                DirectAssociatedYieldOutcome::RecoveryRequired(
                    DirectAssociatedYieldRecovery::classify(association),
                )
            }
        }
    }
}

pub struct DirectYieldedLifecycleEvent {
    _permit: (),
}

impl DirectYieldedLifecycleEvent {
    fn new() -> Self {
        Self { _permit: () }
    }
}

impl DirectIterationAssociation<WorthQueryDirectYieldDenied> {
    pub(in super::super) fn retry(
        self,
    ) -> DirectIterationAssociation<WorthQueryPausedDirectGraphExecution> {
        let Self {
            core,
            graph,
            provider,
            managed,
        } = self;
        DirectIterationAssociation {
            core,
            graph,
            provider,
            managed: managed.into_paused(),
        }
    }
}

impl DirectIterationAssociation<WorthQueryYieldedDirectRun> {
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
