//! Workflow readmission cleanup owns the epoch and lower cleanup lifecycle together.

use super::super::{WorkflowIterationAssociation, WorthQueryConvergenceEpochCore};
use super::{
    WorkflowReadmissionTerminalRecoveryAssociation,
    WorkflowReadmissionYieldReassemblyRecoveryAssociation,
};
use {
    super::super::super::super::super::WorthQueryBoundConvergenceReport,
    super::super::super::super::super::WorthQueryConvergenceEpochCounters,
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryRetainedConvergenceCandidateEvidence,
};
use {
    crate::domain_computation::vocabulary::managed_run::terminal::WorthQueryManagedRunCleanupDisposition,
    crate::domain_computation::WorthQueryReadmissionEvidence,
    crate::domain_computation::WorthQueryWorkflowReadmissionCleanupOutcome,
    crate::domain_computation::WorthQueryWorkflowReadmissionCleanupPending,
    crate::domain_computation::WorthQueryWorkflowReadmissionCleanupRequired,
};

pub(in super::super::super) enum WorkflowAssociatedReadmissionCleanupOutcome {
    Complete(WorkflowReadmissionCleanupReceiptAssociation),
    Pending(WorkflowReadmissionCleanupPendingAssociation),
    RecoveryRequired(WorkflowReadmissionCleanupReceiptAssociation),
}

pub(in super::super::super) struct WorkflowReadmissionCleanupRequiredAssociation {
    core: WorthQueryConvergenceEpochCore,
    managed: WorthQueryWorkflowReadmissionCleanupRequired,
}

pub(in super::super::super) struct WorkflowReadmissionCleanupPendingAssociation {
    core: WorthQueryConvergenceEpochCore,
    managed: WorthQueryWorkflowReadmissionCleanupPending,
}

pub(in super::super::super) struct WorkflowReadmissionCleanupReceiptAssociation {
    core: WorthQueryConvergenceEpochCore,
    disposition: WorthQueryManagedRunCleanupDisposition,
    readmission_evidence: WorthQueryReadmissionEvidence,
}

impl WorkflowReadmissionYieldReassemblyRecoveryAssociation {
    pub(in super::super::super) fn begin_readmission_cleanup(
        self,
    ) -> WorkflowReadmissionCleanupRequiredAssociation {
        let WorkflowIterationAssociation {
            core,
            graph: _,
            provider: _,
            stage_identity: _,
            managed,
        } = self.association;
        WorkflowReadmissionCleanupRequiredAssociation {
            core,
            managed: managed.into_cleanup(),
        }
    }
}

impl WorkflowReadmissionTerminalRecoveryAssociation {
    pub(in super::super::super) fn begin_readmission_cleanup(
        self,
    ) -> WorkflowReadmissionCleanupRequiredAssociation {
        let WorkflowIterationAssociation {
            core,
            graph: _,
            provider: _,
            stage_identity: _,
            managed,
        } = self.association;
        WorkflowReadmissionCleanupRequiredAssociation {
            core,
            managed: managed.into_cleanup(),
        }
    }
}

impl WorkflowReadmissionCleanupRequiredAssociation {
    pub(in super::super::super) fn finish(mut self) -> WorkflowAssociatedReadmissionCleanupOutcome {
        self.core
            .record_lifecycle_event(WorkflowReadmissionCleanupLifecycleEvent::attempted());
        admit_cleanup_outcome(self.core, self.managed.finish())
    }
}

impl WorkflowReadmissionCleanupPendingAssociation {
    pub(in super::super::super) fn retry(mut self) -> WorkflowAssociatedReadmissionCleanupOutcome {
        self.core
            .record_lifecycle_event(WorkflowReadmissionCleanupLifecycleEvent::attempted());
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

impl WorkflowReadmissionCleanupReceiptAssociation {
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
    outcome: WorthQueryWorkflowReadmissionCleanupOutcome,
) -> WorkflowAssociatedReadmissionCleanupOutcome {
    match outcome {
        WorthQueryWorkflowReadmissionCleanupOutcome::Complete(receipt) => {
            core.record_lifecycle_event(WorkflowReadmissionCleanupLifecycleEvent::completed());
            let readmission_evidence = receipt.inspection().readmission_evidence();
            WorkflowAssociatedReadmissionCleanupOutcome::Complete(
                WorkflowReadmissionCleanupReceiptAssociation {
                    core,
                    disposition: WorthQueryManagedRunCleanupDisposition::CleanupComplete,
                    readmission_evidence,
                },
            )
        }
        WorthQueryWorkflowReadmissionCleanupOutcome::Pending(managed) => {
            WorkflowAssociatedReadmissionCleanupOutcome::Pending(
                WorkflowReadmissionCleanupPendingAssociation { core, managed },
            )
        }
        WorthQueryWorkflowReadmissionCleanupOutcome::RecoveryRequired(receipt) => {
            core.record_lifecycle_event(WorkflowReadmissionCleanupLifecycleEvent::completed());
            let readmission_evidence = receipt.inspection().readmission_evidence();
            WorkflowAssociatedReadmissionCleanupOutcome::RecoveryRequired(
                WorkflowReadmissionCleanupReceiptAssociation {
                    core,
                    disposition: WorthQueryManagedRunCleanupDisposition::RecoveryRequired,
                    readmission_evidence,
                },
            )
        }
    }
}

pub(in crate::domain_computation::convergence_epoch) struct WorkflowReadmissionCleanupLifecycleEvent
{
    kind: WorkflowReadmissionCleanupLifecycleEventKind,
}

pub(in crate::domain_computation::convergence_epoch) enum WorkflowReadmissionCleanupLifecycleEventKind
{
    Attempted,
    Completed,
}

impl WorkflowReadmissionCleanupLifecycleEvent {
    fn attempted() -> Self {
        Self {
            kind: WorkflowReadmissionCleanupLifecycleEventKind::Attempted,
        }
    }

    fn completed() -> Self {
        Self {
            kind: WorkflowReadmissionCleanupLifecycleEventKind::Completed,
        }
    }

    pub(in crate::domain_computation::convergence_epoch) fn into_kind(
        self,
    ) -> WorkflowReadmissionCleanupLifecycleEventKind {
        self.kind
    }
}
