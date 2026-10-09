//! Direct readmission transformations that never split associated authority.

use super::DirectIterationAssociation;
use crate::domain_computation::{
    WorthQueryActiveDirectGraphExecution, WorthQueryDirectReadmissionDenied,
    WorthQueryDirectReadmissionOutcome, WorthQueryDirectReadmissionRecoveryRequired,
    WorthQueryDirectReadmissionTerminalRecovery,
    WorthQueryDirectReadmissionYieldReassemblyOutcome as ManagedYieldReassemblyOutcome,
    WorthQueryDirectReadmissionYieldReassemblyRecovery, WorthQueryReadmissionEvidence,
    WorthQueryYieldedDirectRun,
};

mod cleanup;

pub(in super::super) use cleanup::{
    DirectAssociatedReadmissionCleanupOutcome, DirectReadmissionCleanupPendingAssociation,
    DirectReadmissionCleanupReceiptAssociation, DirectReadmissionCleanupRequiredAssociation,
};
pub(in crate::domain_computation::convergence_epoch) use cleanup::{
    DirectReadmissionCleanupLifecycleEvent, DirectReadmissionCleanupLifecycleEventKind,
};

pub(in super::super) struct DirectReadmissionYieldReassemblyRecoveryAssociation {
    association: DirectIterationAssociation<WorthQueryDirectReadmissionYieldReassemblyRecovery>,
}

pub(in super::super) struct DirectReadmissionTerminalRecoveryAssociation {
    association: DirectIterationAssociation<WorthQueryDirectReadmissionTerminalRecovery>,
}

pub(in super::super) enum DirectAssociatedReadmissionOutcome {
    Readmitted {
        association: DirectIterationAssociation<WorthQueryActiveDirectGraphExecution>,
        evidence: WorthQueryReadmissionEvidence,
    },
    Denied(DirectIterationAssociation<WorthQueryDirectReadmissionDenied>),
    RecoveryRequired(DirectAssociatedReadmissionRecovery),
}

pub(in super::super) enum DirectAssociatedReadmissionRecovery {
    YieldReassembly(DirectReadmissionYieldReassemblyRecoveryAssociation),
    TerminalCleanup(DirectReadmissionTerminalRecoveryAssociation),
}

pub(in super::super) enum DirectAssociatedYieldReassemblyOutcome {
    Yielded {
        association: DirectIterationAssociation<WorthQueryYieldedDirectRun>,
        evidence: WorthQueryReadmissionEvidence,
    },
    RecoveryRequired(DirectReadmissionYieldReassemblyRecoveryAssociation),
}

impl DirectIterationAssociation<WorthQueryYieldedDirectRun> {
    pub(in super::super) fn readmit_same_runtime(
        self,
        active_request: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

        query_runtime: &crate::domain_computation::WorthQueryExecutionRuntime,
        bridge_runtime: &worth_runtime_bridge::facade::RuntimeBridge,
    ) -> DirectAssociatedReadmissionOutcome {
        let Self {
            mut core,
            graph,
            provider,
            managed,
        } = self;
        match managed.readmit_same_runtime(active_request, query_runtime, bridge_runtime) {
            WorthQueryDirectReadmissionOutcome::Readmitted(readmitted) => {
                let evidence = readmitted.readmission_evidence();
                core.record_lifecycle_event(DirectReadmittedLifecycleEvent::new());
                DirectAssociatedReadmissionOutcome::Readmitted {
                    association: DirectIterationAssociation {
                        core,
                        graph,
                        provider,
                        managed: readmitted.into_active(),
                    },
                    evidence,
                }
            }
            WorthQueryDirectReadmissionOutcome::Denied(managed) => {
                DirectAssociatedReadmissionOutcome::Denied(DirectIterationAssociation {
                    core,
                    graph,
                    provider,
                    managed,
                })
            }
            WorthQueryDirectReadmissionOutcome::RecoveryRequired(recovery) => {
                let recovery = match recovery {
                    WorthQueryDirectReadmissionRecoveryRequired::YieldReassembly(managed) => {
                        DirectAssociatedReadmissionRecovery::YieldReassembly(
                            DirectReadmissionYieldReassemblyRecoveryAssociation {
                                association: DirectIterationAssociation {
                                    core,
                                    graph,
                                    provider,
                                    managed,
                                },
                            },
                        )
                    }
                    WorthQueryDirectReadmissionRecoveryRequired::TerminalCleanup(managed) => {
                        DirectAssociatedReadmissionRecovery::TerminalCleanup(
                            DirectReadmissionTerminalRecoveryAssociation {
                                association: DirectIterationAssociation {
                                    core,
                                    graph,
                                    provider,
                                    managed,
                                },
                            },
                        )
                    }
                };
                DirectAssociatedReadmissionOutcome::RecoveryRequired(recovery)
            }
        }
    }
}

pub(in crate::domain_computation::convergence_epoch) struct DirectReadmittedLifecycleEvent {
    _permit: (),
}

impl DirectReadmittedLifecycleEvent {
    fn new() -> Self {
        Self { _permit: () }
    }
}

impl DirectIterationAssociation<WorthQueryDirectReadmissionDenied> {
    pub(in super::super) fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.managed.readmission_evidence()
    }

    pub(in super::super) fn into_yielded(
        self,
    ) -> DirectIterationAssociation<WorthQueryYieldedDirectRun> {
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
            managed: managed.into_yielded(),
        }
    }
}

impl DirectReadmissionYieldReassemblyRecoveryAssociation {
    pub(in super::super) fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.association.managed.readmission_evidence()
    }

    pub(in super::super) fn retry_to_yielded(self) -> DirectAssociatedYieldReassemblyOutcome {
        let DirectIterationAssociation {
            core,
            graph,
            provider,
            managed,
        } = self.association;
        match managed.retry_to_yielded() {
            ManagedYieldReassemblyOutcome::Yielded(reassembled) => {
                DirectAssociatedYieldReassemblyOutcome::Yielded {
                    evidence: reassembled.readmission_evidence(),
                    association: DirectIterationAssociation {
                        core,
                        graph,
                        provider,
                        managed: reassembled.into_yielded(),
                    },
                }
            }
            ManagedYieldReassemblyOutcome::RecoveryRequired(managed) => {
                DirectAssociatedYieldReassemblyOutcome::RecoveryRequired(
                    DirectReadmissionYieldReassemblyRecoveryAssociation {
                        association: DirectIterationAssociation {
                            core,
                            graph,
                            provider,
                            managed,
                        },
                    },
                )
            }
        }
    }
}

impl DirectReadmissionTerminalRecoveryAssociation {
    pub(in super::super) fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.association.managed.readmission_evidence()
    }
}
