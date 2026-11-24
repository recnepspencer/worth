use crate::publication::cdc::data::SubscriberCheckpoint;
use serde::{Deserialize, Serialize};
use worth_relational_contracts::internal_port::diagnostics::data::artifacts::RelationalDiagnosticArtifact;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriberStreamFailureClass {
    SchemaUnsupported,
    InvalidBatchSize,
    DurableCoverageGap,
    SubscriberContractMismatch,
    UnsupportedContinuation,
    ContractUpgradeUnsupported,
    RenegotiationRequired,
    DescriptorVersionMismatch,
    CheckpointContinuitySummaryMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriberStreamFailure {
    pub class: SubscriberStreamFailureClass,
    pub detail: String,
    pub latest_available_checkpoint: Option<SubscriberCheckpoint>,
    pub diagnostics: Vec<RelationalDiagnosticArtifact>,
}

impl SubscriberStreamFailure {
    pub(crate) fn new(
        class: SubscriberStreamFailureClass,
        detail: impl Into<String>,
        latest_available_checkpoint: Option<SubscriberCheckpoint>,
        diagnostics: Vec<RelationalDiagnosticArtifact>,
    ) -> Self {
        Self {
            class,
            detail: detail.into(),
            latest_available_checkpoint,
            diagnostics,
        }
    }
}

#[doc(hidden)]
pub trait SubscriberStreamFailureOwnerPort: Sized {
    fn new(
        class: SubscriberStreamFailureClass,
        detail: impl Into<String>,
        latest_available_checkpoint: Option<SubscriberCheckpoint>,
        diagnostics: Vec<RelationalDiagnosticArtifact>,
    ) -> Self;
}

impl SubscriberStreamFailureOwnerPort for SubscriberStreamFailure {
    fn new(
        class: SubscriberStreamFailureClass,
        detail: impl Into<String>,
        latest_available_checkpoint: Option<SubscriberCheckpoint>,
        diagnostics: Vec<RelationalDiagnosticArtifact>,
    ) -> Self {
        Self::new(class, detail, latest_available_checkpoint, diagnostics)
    }
}
