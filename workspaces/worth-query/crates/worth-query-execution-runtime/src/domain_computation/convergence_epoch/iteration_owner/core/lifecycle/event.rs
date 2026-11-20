use super::super::report_history::{ReportHistoryLifecycleEvent, ReportHistoryLifecycleEventKind};
use super::WorthQueryConvergenceEpochCounters;
use crate::domain_computation::convergence_epoch::domain_assessment_transition::{
    DomainAssessmentLifecycleEvent, DomainAssessmentLifecycleEventKind,
};
use crate::domain_computation::convergence_epoch::iteration_owner::{
    direct::{
        DirectAdmissionLifecycleEvent, DirectAdmissionLifecycleEventKind,
        DirectIterationBeganEvent, DirectReadmissionCleanupLifecycleEvent,
        DirectReadmissionCleanupLifecycleEventKind, DirectReadmittedLifecycleEvent,
        DirectTerminalProviderWorkEvent, DirectYieldCleanupLifecycleEvent,
        DirectYieldCleanupLifecycleEventKind, DirectYieldRecoveryCleanupLifecycleEvent,
        DirectYieldRecoveryCleanupLifecycleEventKind, DirectYieldedLifecycleEvent,
    },
    workflow::{
        WorkflowAdmissionLifecycleEvent, WorkflowAdmissionLifecycleEventKind,
        WorkflowIterationBeganEvent, WorkflowReadmissionCleanupLifecycleEvent,
        WorkflowReadmissionCleanupLifecycleEventKind, WorkflowReadmittedLifecycleEvent,
        WorkflowTerminalProviderWorkEvent, WorkflowYieldCleanupLifecycleEvent,
        WorkflowYieldCleanupLifecycleEventKind, WorkflowYieldRecoveryCleanupLifecycleEvent,
        WorkflowYieldRecoveryCleanupLifecycleEventKind, WorkflowYieldedLifecycleEvent,
    },
};
use crate::domain_computation::convergence_epoch::{
    direct_cleanup::{
        DirectTerminalCleanupLifecycleEvent, DirectTerminalCleanupLifecycleEventKind,
    },
    workflow_cleanup::{
        WorkflowTerminalCleanupLifecycleEvent, WorkflowTerminalCleanupLifecycleEventKind,
    },
};

mod sealed {
    pub trait Sealed {}
}

pub trait WorthQueryConvergenceLifecycleEvent:
    sealed::Sealed
{
    fn apply(self, counters: &mut WorthQueryConvergenceEpochCounters);
}

pub trait WorthQueryConvergenceAdmissionStartEvent:
    WorthQueryConvergenceLifecycleEvent
{
}

impl sealed::Sealed for DirectAdmissionLifecycleEvent {}

impl WorthQueryConvergenceLifecycleEvent for DirectAdmissionLifecycleEvent {
    fn apply(self, counters: &mut WorthQueryConvergenceEpochCounters) {
        match self.into_kind() {
            DirectAdmissionLifecycleEventKind::OperationChecked => {
                counters.checked_operation_authority();
            }
            DirectAdmissionLifecycleEventKind::ContractChecked => {
                counters.checked_contract_authority();
            }
            DirectAdmissionLifecycleEventKind::ManagedRunChecked => {
                counters.checked_managed_run_authority();
            }
            DirectAdmissionLifecycleEventKind::GraphChecked => counters.checked_graph_authority(),
        }
    }
}

impl WorthQueryConvergenceAdmissionStartEvent for DirectAdmissionLifecycleEvent {}

impl sealed::Sealed for WorkflowAdmissionLifecycleEvent {}

impl WorthQueryConvergenceLifecycleEvent for WorkflowAdmissionLifecycleEvent {
    fn apply(self, counters: &mut WorthQueryConvergenceEpochCounters) {
        match self.into_kind() {
            WorkflowAdmissionLifecycleEventKind::OperationChecked => {
                counters.checked_operation_authority();
            }
            WorkflowAdmissionLifecycleEventKind::ContractChecked => {
                counters.checked_contract_authority();
            }
            WorkflowAdmissionLifecycleEventKind::ManagedRunChecked => {
                counters.checked_managed_run_authority();
            }
            WorkflowAdmissionLifecycleEventKind::GraphChecked => counters.checked_graph_authority(),
        }
    }
}

impl WorthQueryConvergenceAdmissionStartEvent for WorkflowAdmissionLifecycleEvent {}

macro_rules! impl_began_iteration_event {
    ($event:ty) => {
        impl sealed::Sealed for $event {}

        impl WorthQueryConvergenceLifecycleEvent for $event {
            fn apply(self, counters: &mut WorthQueryConvergenceEpochCounters) {
                counters.began_iteration();
            }
        }
    };
}

impl_began_iteration_event!(DirectIterationBeganEvent);
impl_began_iteration_event!(WorkflowIterationBeganEvent);

macro_rules! impl_terminal_provider_work_event {
    ($event:ty) => {
        impl sealed::Sealed for $event {}

        impl WorthQueryConvergenceLifecycleEvent for $event {
            fn apply(self, counters: &mut WorthQueryConvergenceEpochCounters) {
                counters.reconciled_provider_work_total(self.completed_work_units());
            }
        }
    };
}

impl_terminal_provider_work_event!(DirectTerminalProviderWorkEvent);
impl_terminal_provider_work_event!(WorkflowTerminalProviderWorkEvent);

macro_rules! impl_simple_lifecycle_event {
    ($event:ty, $counter_method:ident) => {
        impl sealed::Sealed for $event {}

        impl WorthQueryConvergenceLifecycleEvent for $event {
            fn apply(self, counters: &mut WorthQueryConvergenceEpochCounters) {
                counters.$counter_method();
            }
        }
    };
}

impl_simple_lifecycle_event!(DirectYieldedLifecycleEvent, yielded);
impl_simple_lifecycle_event!(WorkflowYieldedLifecycleEvent, yielded);
impl_simple_lifecycle_event!(DirectReadmittedLifecycleEvent, readmitted);
impl_simple_lifecycle_event!(WorkflowReadmittedLifecycleEvent, readmitted);

impl sealed::Sealed for DirectTerminalCleanupLifecycleEvent {}

impl WorthQueryConvergenceLifecycleEvent for DirectTerminalCleanupLifecycleEvent {
    fn apply(self, counters: &mut WorthQueryConvergenceEpochCounters) {
        match self.into_kind() {
            DirectTerminalCleanupLifecycleEventKind::Attempted => counters.attempted_cleanup(),
            DirectTerminalCleanupLifecycleEventKind::Completed => counters.completed_cleanup(),
        }
    }
}

impl sealed::Sealed for WorkflowTerminalCleanupLifecycleEvent {}

impl WorthQueryConvergenceLifecycleEvent for WorkflowTerminalCleanupLifecycleEvent {
    fn apply(self, counters: &mut WorthQueryConvergenceEpochCounters) {
        match self.into_kind() {
            WorkflowTerminalCleanupLifecycleEventKind::Attempted => counters.attempted_cleanup(),
            WorkflowTerminalCleanupLifecycleEventKind::Completed => counters.completed_cleanup(),
        }
    }
}

impl sealed::Sealed for DirectYieldCleanupLifecycleEvent {}

impl WorthQueryConvergenceLifecycleEvent for DirectYieldCleanupLifecycleEvent {
    fn apply(self, counters: &mut WorthQueryConvergenceEpochCounters) {
        match self.into_kind() {
            DirectYieldCleanupLifecycleEventKind::Attempted => counters.attempted_cleanup(),
            DirectYieldCleanupLifecycleEventKind::Completed => counters.completed_cleanup(),
        }
    }
}

impl sealed::Sealed for WorkflowYieldCleanupLifecycleEvent {}

impl WorthQueryConvergenceLifecycleEvent for WorkflowYieldCleanupLifecycleEvent {
    fn apply(self, counters: &mut WorthQueryConvergenceEpochCounters) {
        match self.into_kind() {
            WorkflowYieldCleanupLifecycleEventKind::Attempted => counters.attempted_cleanup(),
            WorkflowYieldCleanupLifecycleEventKind::Completed => counters.completed_cleanup(),
        }
    }
}

impl sealed::Sealed for DirectReadmissionCleanupLifecycleEvent {}

impl WorthQueryConvergenceLifecycleEvent for DirectReadmissionCleanupLifecycleEvent {
    fn apply(self, counters: &mut WorthQueryConvergenceEpochCounters) {
        match self.into_kind() {
            DirectReadmissionCleanupLifecycleEventKind::Attempted => counters.attempted_cleanup(),
            DirectReadmissionCleanupLifecycleEventKind::Completed => counters.completed_cleanup(),
        }
    }
}

impl sealed::Sealed for WorkflowReadmissionCleanupLifecycleEvent {}

impl WorthQueryConvergenceLifecycleEvent for WorkflowReadmissionCleanupLifecycleEvent {
    fn apply(self, counters: &mut WorthQueryConvergenceEpochCounters) {
        match self.into_kind() {
            WorkflowReadmissionCleanupLifecycleEventKind::Attempted => {
                counters.attempted_cleanup();
            }
            WorkflowReadmissionCleanupLifecycleEventKind::Completed => {
                counters.completed_cleanup();
            }
        }
    }
}

impl sealed::Sealed for DirectYieldRecoveryCleanupLifecycleEvent {}

impl WorthQueryConvergenceLifecycleEvent for DirectYieldRecoveryCleanupLifecycleEvent {
    fn apply(self, counters: &mut WorthQueryConvergenceEpochCounters) {
        match self.into_kind() {
            DirectYieldRecoveryCleanupLifecycleEventKind::Attempted => {
                counters.attempted_cleanup();
            }
            DirectYieldRecoveryCleanupLifecycleEventKind::Completed => {
                counters.completed_cleanup();
            }
        }
    }
}

impl sealed::Sealed for WorkflowYieldRecoveryCleanupLifecycleEvent {}

impl WorthQueryConvergenceLifecycleEvent for WorkflowYieldRecoveryCleanupLifecycleEvent {
    fn apply(self, counters: &mut WorthQueryConvergenceEpochCounters) {
        match self.into_kind() {
            WorkflowYieldRecoveryCleanupLifecycleEventKind::Attempted => {
                counters.attempted_cleanup();
            }
            WorkflowYieldRecoveryCleanupLifecycleEventKind::Completed => {
                counters.completed_cleanup();
            }
        }
    }
}

impl sealed::Sealed for DomainAssessmentLifecycleEvent {}

impl WorthQueryConvergenceLifecycleEvent for DomainAssessmentLifecycleEvent {
    fn apply(self, counters: &mut WorthQueryConvergenceEpochCounters) {
        match self.into_kind() {
            DomainAssessmentLifecycleEventKind::ProviderWork(completed_work_units) => {
                counters.recorded_provider_work(completed_work_units);
            }
            DomainAssessmentLifecycleEventKind::DomainWork(work) => {
                counters.recorded_domain_work(&work);
            }
        }
    }
}

impl sealed::Sealed for ReportHistoryLifecycleEvent {}

impl WorthQueryConvergenceLifecycleEvent for ReportHistoryLifecycleEvent {
    fn apply(self, counters: &mut WorthQueryConvergenceEpochCounters) {
        match self.into_kind() {
            ReportHistoryLifecycleEventKind::Retained => counters.retained_incumbent(),
            ReportHistoryLifecycleEventKind::IncumbentSetReplaced => {
                counters.replaced_incumbent_set();
            }
        }
    }
}
