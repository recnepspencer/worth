//! Workflow readmission transformations that never split associated authority.

use super::WorkflowIterationAssociation;
use crate::domain_computation::{
    WorthQueryActiveWorkflowGraphExecution, WorthQueryReadmissionEvidence,
    WorthQueryWorkflowReadmissionDenied, WorthQueryWorkflowReadmissionOutcome,
    WorthQueryWorkflowReadmissionRecoveryRequired, WorthQueryWorkflowReadmissionTerminalRecovery,
    WorthQueryWorkflowReadmissionYieldReassemblyOutcome as ManagedYieldReassemblyOutcome,
    WorthQueryWorkflowReadmissionYieldReassemblyRecovery, WorthQueryYieldedWorkflowRun,
};

mod cleanup;

pub(in super::super) use cleanup::{
    WorkflowAssociatedReadmissionCleanupOutcome, WorkflowReadmissionCleanupPendingAssociation,
    WorkflowReadmissionCleanupReceiptAssociation, WorkflowReadmissionCleanupRequiredAssociation,
};
pub(in crate::domain_computation::convergence_epoch) use cleanup::{
    WorkflowReadmissionCleanupLifecycleEvent, WorkflowReadmissionCleanupLifecycleEventKind,
};

pub(in super::super) struct WorkflowReadmissionYieldReassemblyRecoveryAssociation {
    association: WorkflowIterationAssociation<WorthQueryWorkflowReadmissionYieldReassemblyRecovery>,
}

pub(in super::super) struct WorkflowReadmissionTerminalRecoveryAssociation {
    association: WorkflowIterationAssociation<WorthQueryWorkflowReadmissionTerminalRecovery>,
}

pub(in super::super) enum WorkflowAssociatedReadmissionOutcome {
    Readmitted {
        association: WorkflowIterationAssociation<WorthQueryActiveWorkflowGraphExecution>,
        evidence: WorthQueryReadmissionEvidence,
    },
    Denied(WorkflowIterationAssociation<WorthQueryWorkflowReadmissionDenied>),
    RecoveryRequired(WorkflowAssociatedReadmissionRecovery),
}

pub(in super::super) enum WorkflowAssociatedReadmissionRecovery {
    YieldReassembly(WorkflowReadmissionYieldReassemblyRecoveryAssociation),
    TerminalCleanup(WorkflowReadmissionTerminalRecoveryAssociation),
}

pub(in super::super) enum WorkflowAssociatedYieldReassemblyOutcome {
    Yielded {
        association: WorkflowIterationAssociation<WorthQueryYieldedWorkflowRun>,
        evidence: WorthQueryReadmissionEvidence,
    },
    RecoveryRequired(WorkflowReadmissionYieldReassemblyRecoveryAssociation),
}

impl WorkflowIterationAssociation<WorthQueryYieldedWorkflowRun> {
    pub(in super::super) fn readmit_same_runtime(
        self,
        active_request: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

        query_runtime: &crate::domain_computation::WorthQueryExecutionRuntime,
        bridge_runtime: &worth_runtime_bridge::facade::RuntimeBridge,
    ) -> WorkflowAssociatedReadmissionOutcome {
        let Self {
            mut core,
            graph,
            provider,
            stage_identity,
            managed,
        } = self;
        match managed.readmit_same_runtime(active_request, query_runtime, bridge_runtime) {
            WorthQueryWorkflowReadmissionOutcome::Readmitted(readmitted) => {
                let evidence = readmitted.readmission_evidence();
                core.record_lifecycle_event(WorkflowReadmittedLifecycleEvent::new());
                WorkflowAssociatedReadmissionOutcome::Readmitted {
                    association: WorkflowIterationAssociation {
                        core,
                        graph,
                        provider,
                        stage_identity,
                        managed: readmitted.into_active(),
                    },
                    evidence,
                }
            }
            WorthQueryWorkflowReadmissionOutcome::Denied(managed) => {
                WorkflowAssociatedReadmissionOutcome::Denied(WorkflowIterationAssociation {
                    core,
                    graph,
                    provider,
                    stage_identity,
                    managed,
                })
            }
            WorthQueryWorkflowReadmissionOutcome::RecoveryRequired(recovery) => {
                let recovery = match recovery {
                    WorthQueryWorkflowReadmissionRecoveryRequired::YieldReassembly(managed) => {
                        WorkflowAssociatedReadmissionRecovery::YieldReassembly(
                            WorkflowReadmissionYieldReassemblyRecoveryAssociation {
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
                    WorthQueryWorkflowReadmissionRecoveryRequired::TerminalCleanup(managed) => {
                        WorkflowAssociatedReadmissionRecovery::TerminalCleanup(
                            WorkflowReadmissionTerminalRecoveryAssociation {
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
                };
                WorkflowAssociatedReadmissionOutcome::RecoveryRequired(recovery)
            }
        }
    }
}

pub(in crate::domain_computation::convergence_epoch) struct WorkflowReadmittedLifecycleEvent {
    _permit: (),
}

impl WorkflowReadmittedLifecycleEvent {
    fn new() -> Self {
        Self { _permit: () }
    }
}

impl WorkflowIterationAssociation<WorthQueryWorkflowReadmissionDenied> {
    pub(in super::super) fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.managed.readmission_evidence()
    }

    pub(in super::super) fn into_yielded(
        self,
    ) -> WorkflowIterationAssociation<WorthQueryYieldedWorkflowRun> {
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
            managed: managed.into_yielded(),
        }
    }
}

impl WorkflowReadmissionYieldReassemblyRecoveryAssociation {
    pub(in super::super) fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.association.managed.readmission_evidence()
    }

    pub(in super::super) fn retry_to_yielded(self) -> WorkflowAssociatedYieldReassemblyOutcome {
        let WorkflowIterationAssociation {
            core,
            graph,
            provider,
            stage_identity,
            managed,
        } = self.association;
        match managed.retry_to_yielded() {
            ManagedYieldReassemblyOutcome::Yielded(reassembled) => {
                WorkflowAssociatedYieldReassemblyOutcome::Yielded {
                    evidence: reassembled.readmission_evidence(),
                    association: WorkflowIterationAssociation {
                        core,
                        graph,
                        provider,
                        stage_identity,
                        managed: reassembled.into_yielded(),
                    },
                }
            }
            ManagedYieldReassemblyOutcome::RecoveryRequired(managed) => {
                WorkflowAssociatedYieldReassemblyOutcome::RecoveryRequired(
                    WorkflowReadmissionYieldReassemblyRecoveryAssociation {
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
        }
    }
}

impl WorkflowReadmissionTerminalRecoveryAssociation {
    pub(in super::super) fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.association.managed.readmission_evidence()
    }
}
