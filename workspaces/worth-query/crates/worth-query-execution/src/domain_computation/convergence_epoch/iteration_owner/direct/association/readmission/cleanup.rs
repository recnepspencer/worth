//! Direct readmission cleanup owns the epoch and lower cleanup lifecycle together.

use super::super::super::super::super::{
    WorthQueryBoundConvergenceReport, WorthQueryConvergenceEpochCounters,
    WorthQueryRetainedConvergenceCandidateEvidence,
};
use super::super::{DirectIterationAssociation, WorthQueryConvergenceEpochCore};
use super::{
    DirectReadmissionTerminalRecoveryAssociation,
    DirectReadmissionYieldReassemblyRecoveryAssociation,
};
use {
    crate::domain_computation::vocabulary::managed_run::terminal::WorthQueryManagedRunCleanupDisposition,
    crate::domain_computation::WorthQueryDirectReadmissionCleanupOutcome,
    crate::domain_computation::WorthQueryDirectReadmissionCleanupPending,
    crate::domain_computation::WorthQueryDirectReadmissionCleanupRequired,
    crate::domain_computation::WorthQueryReadmissionEvidence,
};

pub(in super::super::super) enum DirectAssociatedReadmissionCleanupOutcome {
    Complete(DirectReadmissionCleanupReceiptAssociation),
    Pending(DirectReadmissionCleanupPendingAssociation),
    RecoveryRequired(DirectReadmissionCleanupReceiptAssociation),
}

pub(in super::super::super) struct DirectReadmissionCleanupRequiredAssociation {
    core: WorthQueryConvergenceEpochCore,
    managed: WorthQueryDirectReadmissionCleanupRequired,
}

pub(in super::super::super) struct DirectReadmissionCleanupPendingAssociation {
    core: WorthQueryConvergenceEpochCore,
    managed: WorthQueryDirectReadmissionCleanupPending,
}

pub(in super::super::super) struct DirectReadmissionCleanupReceiptAssociation {
    core: WorthQueryConvergenceEpochCore,
    disposition: WorthQueryManagedRunCleanupDisposition,
    readmission_evidence: WorthQueryReadmissionEvidence,
}

impl DirectReadmissionYieldReassemblyRecoveryAssociation {
    pub(in super::super::super) fn begin_readmission_cleanup(
        self,
    ) -> DirectReadmissionCleanupRequiredAssociation {
        let DirectIterationAssociation {
            core,
            graph: _,
            provider: _,
            managed,
        } = self.association;
        DirectReadmissionCleanupRequiredAssociation {
            core,
            managed: managed.into_cleanup(),
        }
    }
}

impl DirectReadmissionTerminalRecoveryAssociation {
    pub(in super::super::super) fn begin_readmission_cleanup(
        self,
    ) -> DirectReadmissionCleanupRequiredAssociation {
        let DirectIterationAssociation {
            core,
            graph: _,
            provider: _,
            managed,
        } = self.association;
        DirectReadmissionCleanupRequiredAssociation {
            core,
            managed: managed.into_cleanup(),
        }
    }
}

impl DirectReadmissionCleanupRequiredAssociation {
    pub(in super::super::super) fn finish(mut self) -> DirectAssociatedReadmissionCleanupOutcome {
        self.core
            .record_lifecycle_event(DirectReadmissionCleanupLifecycleEvent::attempted());
        admit_cleanup_outcome(self.core, self.managed.finish())
    }
}

impl DirectReadmissionCleanupPendingAssociation {
    pub(in super::super::super) fn retry(mut self) -> DirectAssociatedReadmissionCleanupOutcome {
        self.core
            .record_lifecycle_event(DirectReadmissionCleanupLifecycleEvent::attempted());
        admit_cleanup_outcome(self.core, self.managed.retry())
    }

    pub(in super::super::super) fn epoch_identity(&self) -> &str {
        self.core.identity()
    }

    pub(in super::super::super) fn counters(&self) -> &WorthQueryConvergenceEpochCounters {
        self.core.counters()
    }

    pub(in super::super::super) fn incumbents(
        &self,
    ) -> &[WorthQueryRetainedConvergenceCandidateEvidence] {
        self.core.incumbents()
    }

    pub(in super::super::super) fn latest_report(
        &self,
    ) -> Option<&WorthQueryBoundConvergenceReport> {
        self.core.latest_report()
    }

    pub(in super::super::super) fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.managed.inspection().readmission_evidence()
    }
}

impl DirectReadmissionCleanupReceiptAssociation {
    pub(in super::super::super) fn epoch_identity(&self) -> &str {
        self.core.identity()
    }

    pub(in super::super::super) fn counters(&self) -> &WorthQueryConvergenceEpochCounters {
        self.core.counters()
    }

    pub(in super::super::super) fn incumbents(
        &self,
    ) -> &[WorthQueryRetainedConvergenceCandidateEvidence] {
        self.core.incumbents()
    }

    pub(in super::super::super) fn latest_report(
        &self,
    ) -> Option<&WorthQueryBoundConvergenceReport> {
        self.core.latest_report()
    }

    pub(in super::super::super) const fn disposition(
        &self,
    ) -> WorthQueryManagedRunCleanupDisposition {
        self.disposition
    }

    pub(in super::super::super) const fn readmission_evidence(
        &self,
    ) -> WorthQueryReadmissionEvidence {
        self.readmission_evidence
    }
}

fn admit_cleanup_outcome(
    mut core: WorthQueryConvergenceEpochCore,
    outcome: WorthQueryDirectReadmissionCleanupOutcome,
) -> DirectAssociatedReadmissionCleanupOutcome {
    match outcome {
        WorthQueryDirectReadmissionCleanupOutcome::Complete(receipt) => {
            core.record_lifecycle_event(DirectReadmissionCleanupLifecycleEvent::completed());
            let readmission_evidence = receipt.inspection().readmission_evidence();
            DirectAssociatedReadmissionCleanupOutcome::Complete(
                DirectReadmissionCleanupReceiptAssociation {
                    core,
                    disposition: WorthQueryManagedRunCleanupDisposition::CleanupComplete,
                    readmission_evidence,
                },
            )
        }
        WorthQueryDirectReadmissionCleanupOutcome::Pending(managed) => {
            DirectAssociatedReadmissionCleanupOutcome::Pending(
                DirectReadmissionCleanupPendingAssociation { core, managed },
            )
        }
        WorthQueryDirectReadmissionCleanupOutcome::RecoveryRequired(receipt) => {
            core.record_lifecycle_event(DirectReadmissionCleanupLifecycleEvent::completed());
            let readmission_evidence = receipt.inspection().readmission_evidence();
            DirectAssociatedReadmissionCleanupOutcome::RecoveryRequired(
                DirectReadmissionCleanupReceiptAssociation {
                    core,
                    disposition: WorthQueryManagedRunCleanupDisposition::RecoveryRequired,
                    readmission_evidence,
                },
            )
        }
    }
}

pub(in crate::domain_computation::convergence_epoch) struct DirectReadmissionCleanupLifecycleEvent {
    kind: DirectReadmissionCleanupLifecycleEventKind,
}

pub(in crate::domain_computation::convergence_epoch) enum DirectReadmissionCleanupLifecycleEventKind
{
    Attempted,
    Completed,
}

impl DirectReadmissionCleanupLifecycleEvent {
    fn attempted() -> Self {
        Self {
            kind: DirectReadmissionCleanupLifecycleEventKind::Attempted,
        }
    }

    fn completed() -> Self {
        Self {
            kind: DirectReadmissionCleanupLifecycleEventKind::Completed,
        }
    }

    pub(in crate::domain_computation::convergence_epoch) fn into_kind(
        self,
    ) -> DirectReadmissionCleanupLifecycleEventKind {
        self.kind
    }
}
