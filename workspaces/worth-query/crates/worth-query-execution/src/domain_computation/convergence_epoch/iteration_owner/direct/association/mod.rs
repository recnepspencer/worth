//! Move-only association of every authority axis in a direct iteration.

use std::sync::Arc;

use worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority;

use super::super::super::{
    WorthQueryConvergenceDomainProvider, WorthQueryConvergenceEpochCounters,
    WorthQueryConvergenceEpochDenial, WorthQueryConvergenceEpochDenialKind as Kind,
    WorthQueryConvergenceIndeterminateCause,
    WorthQueryConvergenceIterationStartFailureKind as StartFailureKind,
    WorthQueryConvergenceTerminalKind,
};
use super::super::core::WorthQueryConvergenceEpochCore;
use crate::domain_computation::{
    WorthQueryActiveDirectGraphExecution, WorthQueryAdmittedDirectRun,
    WorthQueryManagedGraphCallRequest, WorthQueryManagedRunTerminalKind,
    WorthQueryRunningDirectRun,
};

mod admit;
mod completion;
mod readmission;
mod step;
mod yield_cleanup;
mod yield_phase;

pub(in crate::domain_computation::convergence_epoch) use admit::{
    admit_epoch, DirectAdmissionLifecycleEvent, DirectAdmissionLifecycleEventKind,
};
pub(in crate::domain_computation::convergence_epoch) use completion::DirectTerminalProviderWorkEvent;
pub(super) use readmission::{
    DirectAssociatedReadmissionCleanupOutcome, DirectAssociatedReadmissionOutcome,
    DirectAssociatedReadmissionRecovery, DirectAssociatedYieldReassemblyOutcome,
    DirectReadmissionCleanupPendingAssociation, DirectReadmissionCleanupReceiptAssociation,
    DirectReadmissionCleanupRequiredAssociation, DirectReadmissionTerminalRecoveryAssociation,
    DirectReadmissionYieldReassemblyRecoveryAssociation,
};
pub(in crate::domain_computation::convergence_epoch) use readmission::{
    DirectReadmissionCleanupLifecycleEvent, DirectReadmissionCleanupLifecycleEventKind,
    DirectReadmittedLifecycleEvent,
};
pub(super) use step::DirectAssociatedStepOutcome;
pub(super) use yield_cleanup::{
    DirectAssociatedYieldCleanupOutcome, DirectYieldCleanupReceiptAssociation,
};
pub(in crate::domain_computation::convergence_epoch) use yield_cleanup::{
    DirectYieldCleanupLifecycleEvent, DirectYieldCleanupLifecycleEventKind,
};
pub(super) use yield_phase::{
    DirectAssociatedYieldOutcome, DirectAssociatedYieldRecovery,
    DirectYieldRecoveryCleanupReceiptAssociation, DirectYieldRunningRecoveryAssociation,
    DirectYieldTerminalCleanupAssociation,
};
pub(in crate::domain_computation::convergence_epoch) use yield_phase::{
    DirectYieldRecoveryCleanupLifecycleEvent, DirectYieldRecoveryCleanupLifecycleEventKind,
    DirectYieldedLifecycleEvent,
};

pub(super) struct DirectIterationAssociation<Managed> {
    core: WorthQueryConvergenceEpochCore,
    graph: WorthQueryInstalledGraphParticipationAuthority,
    provider: Arc<dyn WorthQueryConvergenceDomainProvider>,
    managed: Managed,
}

pub(super) struct DirectAdmittedEpochAssociation {
    core: WorthQueryConvergenceEpochCore,
    managed_run: WorthQueryAdmittedDirectRun,
    graph: WorthQueryInstalledGraphParticipationAuthority,
    provider: Arc<dyn WorthQueryConvergenceDomainProvider>,
}

pub(super) struct DirectIteratingEpochAssociation {
    core: WorthQueryConvergenceEpochCore,
    managed_run: WorthQueryRunningDirectRun,
    graph: WorthQueryInstalledGraphParticipationAuthority,
    provider: Arc<dyn WorthQueryConvergenceDomainProvider>,
}

pub(super) struct DirectAssociatedStartRejection {
    denial: WorthQueryConvergenceEpochDenial,
    epoch: DirectIteratingEpochAssociation,
}

impl DirectAdmittedEpochAssociation {
    pub(super) fn identity(&self) -> &str {
        self.core.identity()
    }

    pub(super) fn counters(&self) -> &WorthQueryConvergenceEpochCounters {
        self.core.counters()
    }

    pub(super) fn start(self) -> DirectIteratingEpochAssociation {
        DirectIteratingEpochAssociation {
            core: self.core,
            managed_run: self.managed_run.start(),
            graph: self.graph,
            provider: self.provider,
        }
    }
}

impl DirectIteratingEpochAssociation {
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

impl DirectAssociatedStartRejection {
    pub(super) fn denial(&self) -> &WorthQueryConvergenceEpochDenial {
        &self.denial
    }

    pub(super) fn into_epoch(self) -> DirectIteratingEpochAssociation {
        self.epoch
    }

    pub(super) fn terminate(self) -> super::WorthQueryDirectConvergenceIterationStartTermination {
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
            super::super::super::terminal_outcome::direct_terminal_outcome(
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

impl DirectIterationAssociation<WorthQueryActiveDirectGraphExecution> {
    pub(super) fn begin(
        execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        epoch: DirectIteratingEpochAssociation,
        request: WorthQueryManagedGraphCallRequest,
    ) -> Result<Self, DirectAssociatedStartRejection> {
        if epoch.core.counters().iteration_count() >= epoch.core.contract().iteration_bound() {
            return Err(start_rejection(
                epoch,
                Kind::IterationBudgetExhausted,
                Arc::from("convergence iteration budget is already exhausted"),
            ));
        }
        let DirectIteratingEpochAssociation {
            mut core,
            managed_run,
            graph,
            provider,
        } = epoch;
        let managed = match managed_run.begin_graph_execution(execution, &graph, request) {
            Ok(managed) => managed,
            Err(failure) => {
                let detail = Arc::from(failure.detail());
                let kind = failure.kind();
                return Err(start_rejection(
                    DirectIteratingEpochAssociation {
                        core,
                        managed_run: failure.into_running(),
                        graph,
                        provider,
                    },
                    Kind::ManagedIterationStart(StartFailureKind::Direct(kind)),
                    detail,
                ));
            }
        };
        core.record_lifecycle_event(DirectIterationBeganEvent::new());
        Ok(Self {
            core,
            graph,
            provider,
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

pub(in crate::domain_computation::convergence_epoch) struct DirectIterationBeganEvent {
    _permit: (),
}

impl DirectIterationBeganEvent {
    fn new() -> Self {
        Self { _permit: () }
    }
}

fn start_rejection(
    epoch: DirectIteratingEpochAssociation,
    kind: Kind,
    detail: Arc<str>,
) -> DirectAssociatedStartRejection {
    DirectAssociatedStartRejection {
        denial: WorthQueryConvergenceEpochDenial::new(kind, detail, epoch.core.counters().clone()),
        epoch,
    }
}
