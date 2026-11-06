use std::sync::Arc;

use {
    crate::domain_computation::vocabulary::managed_run::terminal::WorthQueryManagedRunTerminalKind,
    crate::domain_computation::WorthQueryConvergenceDomainEvidenceBindingDenial,
    crate::domain_computation::WorthQueryManagedRunDenial,
};

use {
    super::WorthQueryConvergenceEpochDenial,
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceDomainWorkEvidence,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConvergenceDomainPhase {
    Comparator,
    ProgressMeasure,
    RepeatedStateDetector,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConvergenceDomainInvocationFailureKind {
    Rejected,
    Panicked,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryConvergenceDomainInvocationFailure {
    phase: WorthQueryConvergenceDomainPhase,
    kind: WorthQueryConvergenceDomainInvocationFailureKind,
    detail: Arc<str>,
    work: WorthQueryConvergenceDomainWorkEvidence,
}

impl WorthQueryConvergenceDomainInvocationFailure {
    pub(super) fn new(
        phase: WorthQueryConvergenceDomainPhase,
        kind: WorthQueryConvergenceDomainInvocationFailureKind,
        detail: impl Into<Arc<str>>,
        work: WorthQueryConvergenceDomainWorkEvidence,
    ) -> Self {
        Self {
            phase,
            kind,
            detail: detail.into(),
            work,
        }
    }

    pub const fn phase(&self) -> WorthQueryConvergenceDomainPhase {
        self.phase
    }

    pub const fn kind(&self) -> WorthQueryConvergenceDomainInvocationFailureKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub const fn work(&self) -> WorthQueryConvergenceDomainWorkEvidence {
        self.work
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryConvergenceIndeterminateCause {
    DomainInvocation(WorthQueryConvergenceDomainInvocationFailure),
    DomainEvidenceBinding(WorthQueryConvergenceDomainEvidenceBindingDenial),
    ReportAdmission(WorthQueryConvergenceEpochDenial),
    ManagedCompletion(WorthQueryManagedRunDenial),
    ManagedTerminal(WorthQueryManagedRunTerminalKind),
    EpochProgression(WorthQueryConvergenceEpochDenial),
}
