use std::sync::Arc;

use super::super::WorthQueryYieldedDirectRun;
use super::evidence::WorthQueryReadmissionEvidence;
use super::readmitted_execution::WorthQueryReadmittedDirectGraphExecution;
use super::recovery::WorthQueryDirectReadmissionRecoveryRequired;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDirectReadmissionDenialKind {
    /// The phase was lent by another installed runtime.
    ForeignAdvancementPhase,
    ForeignQueryRuntime,
    StaleInstallationGeneration,
    RetainedCapacityMismatch,
    RelationalLeaseNotLive,
    ProviderCheckpointMismatch,
    BridgeReadmissionDenied,
    ProviderCallBindingDenied,
    ProviderStepContractDenied(super::super::WorthQueryManagedStepContractDenialKind),
    ProviderRestoreDenied,
}

#[must_use = "direct readmission outcomes retain running, yielded, or recovery authority"]
pub enum WorthQueryDirectReadmissionOutcome {
    Readmitted(WorthQueryReadmittedDirectGraphExecution),
    Denied(WorthQueryDirectReadmissionDenied),
    RecoveryRequired(WorthQueryDirectReadmissionRecoveryRequired),
}

#[must_use = "direct readmission denial retains the yielded run capability"]
pub struct WorthQueryDirectReadmissionDenied {
    kind: WorthQueryDirectReadmissionDenialKind,
    detail: Arc<str>,
    yielded: WorthQueryYieldedDirectRun,
    evidence: WorthQueryReadmissionEvidence,
}

impl WorthQueryDirectReadmissionDenied {
    pub(super) fn new(
        kind: WorthQueryDirectReadmissionDenialKind,
        detail: impl Into<Arc<str>>,
        yielded: WorthQueryYieldedDirectRun,
        evidence: WorthQueryReadmissionEvidence,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
            yielded,
            evidence,
        }
    }

    pub const fn kind(&self) -> WorthQueryDirectReadmissionDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub const fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.evidence
    }

    pub fn into_yielded(self) -> WorthQueryYieldedDirectRun {
        self.yielded
    }
}
