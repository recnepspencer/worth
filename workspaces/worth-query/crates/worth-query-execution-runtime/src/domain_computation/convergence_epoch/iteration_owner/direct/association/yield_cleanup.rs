//! Direct yielded-run cleanup under one sealed convergence association.

use super::{DirectIterationAssociation, WorthQueryConvergenceEpochCore};
use crate::domain_computation::{WorthQueryDirectYieldCleanupOutcome, WorthQueryYieldedDirectRun};

pub(in super::super) enum DirectAssociatedYieldCleanupOutcome {
    Complete(DirectYieldCleanupReceiptAssociation),
    RecoveryRequired(DirectYieldCleanupReceiptAssociation),
}

pub(in super::super) struct DirectYieldCleanupReceiptAssociation {
    core: WorthQueryConvergenceEpochCore,
}

impl DirectIterationAssociation<WorthQueryYieldedDirectRun> {
    pub(in super::super) fn cleanup(self) -> DirectAssociatedYieldCleanupOutcome {
        let Self {
            mut core,
            graph: _,
            provider: _,
            managed,
        } = self;
        core.record_lifecycle_event(DirectYieldCleanupLifecycleEvent::attempted());
        match managed.cleanup() {
            WorthQueryDirectYieldCleanupOutcome::Complete(_closed_receipt) => {
                core.record_lifecycle_event(DirectYieldCleanupLifecycleEvent::completed());
                DirectAssociatedYieldCleanupOutcome::Complete(
                    DirectYieldCleanupReceiptAssociation { core },
                )
            }
            WorthQueryDirectYieldCleanupOutcome::RecoveryRequired(_closed_receipt) => {
                core.record_lifecycle_event(DirectYieldCleanupLifecycleEvent::completed());
                DirectAssociatedYieldCleanupOutcome::RecoveryRequired(
                    DirectYieldCleanupReceiptAssociation { core },
                )
            }
        }
    }
}

pub struct DirectYieldCleanupLifecycleEvent {
    kind: DirectYieldCleanupLifecycleEventKind,
}

pub enum DirectYieldCleanupLifecycleEventKind {
    Attempted,
    Completed,
}

impl DirectYieldCleanupLifecycleEvent {
    fn attempted() -> Self {
        Self {
            kind: DirectYieldCleanupLifecycleEventKind::Attempted,
        }
    }

    fn completed() -> Self {
        Self {
            kind: DirectYieldCleanupLifecycleEventKind::Completed,
        }
    }

    pub fn into_kind(
        self,
    ) -> DirectYieldCleanupLifecycleEventKind {
        self.kind
    }
}

impl DirectYieldCleanupReceiptAssociation {
    pub(in super::super) fn core(&self) -> &WorthQueryConvergenceEpochCore {
        &self.core
    }
}
