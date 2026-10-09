use std::sync::Arc;

use super::super::WorthQueryYieldedWorkflowRun;
use super::evidence::WorthQueryReadmissionEvidence;
use super::readmitted_execution::WorthQueryReadmittedWorkflowGraphExecution;
use super::recovery::WorthQueryWorkflowReadmissionRecoveryRequired;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowReadmissionDenialKind {
    /// The phase was lent by another installed runtime.
    ForeignAdvancementPhase,
    ForeignQueryRuntime,
    StaleInstallationGeneration,
    RetainedCapacityMismatch,
    RelationalLeaseNotLive,
    ProviderCheckpointMismatch,
    ArtifactGenerationMismatch,
    BridgeReadmissionDenied,
    WorkflowStageResourcesUnavailable,
    ProviderCallBindingDenied,
    ProviderStepContractDenied(super::super::WorthQueryManagedStepContractDenialKind),
    ProviderRestoreDenied,
    ArtifactGenerationDenied,
    ArtifactAuthorityDenied,
}

#[must_use = "workflow readmission outcomes retain running, yielded, or recovery authority"]
pub enum WorthQueryWorkflowReadmissionOutcome {
    Readmitted(WorthQueryReadmittedWorkflowGraphExecution),
    Denied(WorthQueryWorkflowReadmissionDenied),
    RecoveryRequired(WorthQueryWorkflowReadmissionRecoveryRequired),
}

#[must_use = "workflow readmission denial retains the yielded run capability"]
pub struct WorthQueryWorkflowReadmissionDenied {
    kind: WorthQueryWorkflowReadmissionDenialKind,
    detail: Arc<str>,
    yielded: WorthQueryYieldedWorkflowRun,
    evidence: WorthQueryReadmissionEvidence,
}

impl WorthQueryWorkflowReadmissionDenied {
    pub(super) fn new(
        kind: WorthQueryWorkflowReadmissionDenialKind,
        detail: impl Into<Arc<str>>,
        yielded: WorthQueryYieldedWorkflowRun,
        evidence: WorthQueryReadmissionEvidence,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
            yielded,
            evidence,
        }
    }

    pub const fn kind(&self) -> WorthQueryWorkflowReadmissionDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub const fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.evidence
    }

    pub fn into_yielded(self) -> WorthQueryYieldedWorkflowRun {
        self.yielded
    }
}
