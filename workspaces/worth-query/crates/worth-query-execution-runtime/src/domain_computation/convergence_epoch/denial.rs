use std::sync::Arc;

use super::WorthQueryConvergenceEpochCounters;
use crate::domain_computation::managed_run::{
    WorthQueryDirectGraphExecutionStartFailureKind,
    WorthQueryWorkflowGraphExecutionStartFailureKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConvergenceIterationStartFailureKind {
    Direct(WorthQueryDirectGraphExecutionStartFailureKind),
    Workflow(WorthQueryWorkflowGraphExecutionStartFailureKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConvergenceEpochDenialKind {
    ForeignQueryRuntime,
    StaleInstallationGeneration,
    ContractOperationMismatch,
    ManagedRunOperationMismatch,
    GraphOperationMismatch,
    MissingConvergenceProvider,
    ConvergenceProviderFamilyInspectionPanicked,
    ConvergenceProviderFamilyMismatch,
    WorkflowEvidenceStageMismatch,
    IterationBudgetExhausted,
    ManagedIterationStart(WorthQueryConvergenceIterationStartFailureKind),
    IterationRunMismatch,
    DomainEvidenceBinding,
    InvalidDomainReport,
    InvalidIncumbentTransition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryConvergenceEpochDenial {
    kind: WorthQueryConvergenceEpochDenialKind,
    detail: Arc<str>,
    counters: WorthQueryConvergenceEpochCounters,
}

impl WorthQueryConvergenceEpochDenial {
    pub(super) fn new(
        kind: WorthQueryConvergenceEpochDenialKind,
        detail: impl Into<Arc<str>>,
        counters: WorthQueryConvergenceEpochCounters,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
            counters,
        }
    }

    pub const fn kind(&self) -> WorthQueryConvergenceEpochDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn counters(&self) -> &WorthQueryConvergenceEpochCounters {
        &self.counters
    }
}
