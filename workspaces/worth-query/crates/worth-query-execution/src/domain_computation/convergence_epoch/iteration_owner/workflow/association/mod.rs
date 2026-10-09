//! Move-only association of every authority axis in a workflow iteration.

use std::sync::Arc;

use worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority;

use super::super::super::{
    WorthQueryConvergenceDomainProvider, WorthQueryConvergenceEpochCounters,
    WorthQueryConvergenceEpochDenial, WorthQueryConvergenceEpochDenialKind as Kind,
    WorthQueryConvergenceIndeterminateCause, WorthQueryConvergenceIterationStartFailureKind,
    WorthQueryConvergenceTerminalKind,
};
use super::super::core::WorthQueryConvergenceEpochCore;
use crate::domain_computation::{
    WorthQueryActiveWorkflowGraphExecution, WorthQueryAdmittedWorkflowRun,
    WorthQueryManagedGraphCallRequest, WorthQueryManagedRunTerminalKind,
    WorthQueryRunningWorkflowRun, WorthQueryWorkflowRunStartRejection,
};

mod admit;
mod completion;
mod readmission;
mod step;
mod yield_cleanup;
mod yield_phase;

pub(in crate::domain_computation::convergence_epoch) use admit::{
    admit_epoch, WorkflowAdmissionLifecycleEvent, WorkflowAdmissionLifecycleEventKind,
};
pub(in crate::domain_computation::convergence_epoch) use completion::WorkflowTerminalProviderWorkEvent;
pub(super) use readmission::{
    WorkflowAssociatedReadmissionCleanupOutcome, WorkflowAssociatedReadmissionOutcome,
    WorkflowAssociatedReadmissionRecovery, WorkflowAssociatedYieldReassemblyOutcome,
    WorkflowReadmissionCleanupPendingAssociation, WorkflowReadmissionCleanupReceiptAssociation,
    WorkflowReadmissionCleanupRequiredAssociation, WorkflowReadmissionTerminalRecoveryAssociation,
    WorkflowReadmissionYieldReassemblyRecoveryAssociation,
};
pub(in crate::domain_computation::convergence_epoch) use readmission::{
    WorkflowReadmissionCleanupLifecycleEvent, WorkflowReadmissionCleanupLifecycleEventKind,
    WorkflowReadmittedLifecycleEvent,
};
pub(super) use step::WorkflowAssociatedStepOutcome;
pub(super) use yield_cleanup::{
    WorkflowAssociatedYieldCleanupOutcome, WorkflowYieldCleanupPendingAssociation,
    WorkflowYieldCleanupReceiptAssociation,
};
pub(in crate::domain_computation::convergence_epoch) use yield_cleanup::{
    WorkflowYieldCleanupLifecycleEvent, WorkflowYieldCleanupLifecycleEventKind,
};
pub(super) use yield_phase::{
    WorkflowAssociatedYieldOutcome, WorkflowAssociatedYieldRecovery,
    WorkflowAssociatedYieldRecoveryCleanupOutcome, WorkflowYieldRecoveryCleanupPendingAssociation,
    WorkflowYieldRecoveryCleanupReceiptAssociation, WorkflowYieldRunningRecoveryAssociation,
    WorkflowYieldTerminalCleanupAssociation,
};
pub(in crate::domain_computation::convergence_epoch) use yield_phase::{
    WorkflowYieldRecoveryCleanupLifecycleEvent, WorkflowYieldRecoveryCleanupLifecycleEventKind,
    WorkflowYieldedLifecycleEvent,
};

pub(super) struct WorkflowIterationAssociation<Managed> {
    core: WorthQueryConvergenceEpochCore,
    graph: WorthQueryInstalledGraphParticipationAuthority,
    provider: Arc<dyn WorthQueryConvergenceDomainProvider>,
    stage_identity: Arc<str>,
    managed: Managed,
}

pub(super) struct WorkflowAdmittedEpochAssociation {
    core: WorthQueryConvergenceEpochCore,
    managed_run: WorthQueryAdmittedWorkflowRun,
    graph: WorthQueryInstalledGraphParticipationAuthority,
    provider: Arc<dyn WorthQueryConvergenceDomainProvider>,
}

pub(super) struct WorkflowIteratingEpochAssociation {
    core: WorthQueryConvergenceEpochCore,
    managed_run: WorthQueryRunningWorkflowRun,
    graph: WorthQueryInstalledGraphParticipationAuthority,
    provider: Arc<dyn WorthQueryConvergenceDomainProvider>,
}

pub(super) struct WorkflowStartRejectionAssociation {
    core: WorthQueryConvergenceEpochCore,
    rejection: WorthQueryWorkflowRunStartRejection,
    graph: WorthQueryInstalledGraphParticipationAuthority,
    provider: Arc<dyn WorthQueryConvergenceDomainProvider>,
}

pub(super) struct WorkflowAssociatedIterationStartRejection {
    denial: WorthQueryConvergenceEpochDenial,
    epoch: WorkflowIteratingEpochAssociation,
}

impl WorkflowAdmittedEpochAssociation {
    pub(super) fn identity(&self) -> &str {
        self.core.identity()
    }

    pub(super) fn counters(&self) -> &WorthQueryConvergenceEpochCounters {
        self.core.counters()
    }

    pub(super) fn start(
        self,
    ) -> Result<WorkflowIteratingEpochAssociation, WorkflowStartRejectionAssociation> {
        match self.managed_run.start() {
            Ok(managed_run) => Ok(WorkflowIteratingEpochAssociation {
                core: self.core,
                managed_run,
                graph: self.graph,
                provider: self.provider,
            }),
            Err(rejection) => Err(WorkflowStartRejectionAssociation {
                core: self.core,
                rejection,
                graph: self.graph,
                provider: self.provider,
            }),
        }
    }
}

impl WorkflowStartRejectionAssociation {
    pub(super) fn managed_run_rejection(&self) -> &WorthQueryWorkflowRunStartRejection {
        &self.rejection
    }

    pub(super) fn into_admitted(self) -> WorkflowAdmittedEpochAssociation {
        WorkflowAdmittedEpochAssociation {
            core: self.core,
            managed_run: self.rejection.into_admitted(),
            graph: self.graph,
            provider: self.provider,
        }
    }
}

impl WorkflowIteratingEpochAssociation {
    pub(super) fn identity(&self) -> &str {
        self.core.identity()
    }

    pub(super) fn logical_run_identity(&self) -> &str {
        self.core.logical_run_identity()
    }

    pub(super) fn counters(&self) -> &WorthQueryConvergenceEpochCounters {
        self.core.counters()
    }
}

impl WorkflowAssociatedIterationStartRejection {
    pub(super) fn denial(&self) -> &WorthQueryConvergenceEpochDenial {
        &self.denial
    }

    pub(super) fn into_epoch(self) -> WorkflowIteratingEpochAssociation {
        self.epoch
    }

    pub(super) fn terminate(self) -> super::WorthQueryWorkflowConvergenceIterationStartTermination {
        let terminal_kind = if self.denial.kind() == Kind::IterationBudgetExhausted {
            WorthQueryConvergenceTerminalKind::Exhausted
        } else {
            WorthQueryConvergenceTerminalKind::Indeterminate
        };
        let managed_kind = if terminal_kind == WorthQueryConvergenceTerminalKind::Exhausted {
            WorthQueryManagedRunTerminalKind::Exhausted
        } else {
            WorthQueryManagedRunTerminalKind::Failed
        };
        let cause =
            (terminal_kind == WorthQueryConvergenceTerminalKind::Indeterminate).then(|| {
                WorthQueryConvergenceIndeterminateCause::EpochProgression(self.denial.clone())
            });
        super::epoch::start_termination(
            self.denial,
            super::super::super::terminal_outcome::workflow_terminal_outcome(
                self.epoch.core,
                self.epoch
                    .managed_run
                    .terminate_for_convergence(managed_kind),
                terminal_kind,
                cause,
            ),
        )
    }
}

impl WorkflowIterationAssociation<WorthQueryActiveWorkflowGraphExecution> {
    pub(super) fn begin(
        execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        epoch: WorkflowIteratingEpochAssociation,
        stage_identity: &str,
        request: WorthQueryManagedGraphCallRequest,
    ) -> Result<Self, WorkflowAssociatedIterationStartRejection> {
        if epoch.core.counters().iteration_count() >= epoch.core.contract().iteration_bound() {
            return Err(iteration_start_rejection(
                epoch,
                Kind::IterationBudgetExhausted,
                Arc::from("convergence iteration budget is already exhausted"),
            ));
        }
        if epoch.core.contract().evidence_stage_identity() != Some(stage_identity) {
            return Err(iteration_start_rejection(
                epoch,
                Kind::WorkflowEvidenceStageMismatch,
                Arc::from(
                    "workflow convergence iteration must use the installed evidence stage sealed by admission",
                ),
            ));
        }
        let WorkflowIteratingEpochAssociation {
            mut core,
            managed_run,
            graph,
            provider,
        } = epoch;
        let managed = match managed_run.begin_stage_graph_execution(
            execution,
            stage_identity,
            &graph,
            request,
        ) {
            Ok(managed) => managed,
            Err(failure) => {
                let detail = Arc::from(failure.detail());
                let kind = failure.kind();
                return Err(iteration_start_rejection(
                    WorkflowIteratingEpochAssociation {
                        core,
                        managed_run: failure.into_running(),
                        graph,
                        provider,
                    },
                    Kind::ManagedIterationStart(
                        WorthQueryConvergenceIterationStartFailureKind::Workflow(kind),
                    ),
                    detail,
                ));
            }
        };
        core.record_lifecycle_event(WorkflowIterationBeganEvent::new());
        Ok(Self {
            core,
            graph,
            provider,
            stage_identity: Arc::from(stage_identity),
            managed,
        })
    }

    pub(super) fn epoch_identity(&self) -> &str {
        self.core.identity()
    }

    pub(super) fn request_cancellation(
        &self,
        reason: worth_runtime_bridge::facade::BridgeManagedExecutionCancellationReason,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeManagedExecutionCancellation,
        worth_runtime_bridge::facade::BridgeManagedExecutionInterruptionFailure,
    > {
        self.managed.request_cancellation(reason)
    }
}

pub(in crate::domain_computation::convergence_epoch) struct WorkflowIterationBeganEvent {
    _permit: (),
}

impl WorkflowIterationBeganEvent {
    fn new() -> Self {
        Self { _permit: () }
    }
}

fn iteration_start_rejection(
    epoch: WorkflowIteratingEpochAssociation,
    kind: Kind,
    detail: Arc<str>,
) -> WorkflowAssociatedIterationStartRejection {
    WorkflowAssociatedIterationStartRejection {
        denial: WorthQueryConvergenceEpochDenial::new(kind, detail, epoch.core.counters().clone()),
        epoch,
    }
}
