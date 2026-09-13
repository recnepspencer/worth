use std::sync::Arc;

use crate::domain_computation::WorthQueryWorkflowExecutionResourceAttempt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedWorkflowRunAdmissionFailureKind {
    QueryAuthority,
    ProductBasisMismatch,
    RelationalBasis,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    BridgePlanning,
    InstalledStepContract,
    BridgeExecutionBasis,
    ManagedAuthorityJoin,
}

pub struct WorthQueryManagedWorkflowRunAdmissionFailure {
    kind: WorthQueryManagedWorkflowRunAdmissionFailureKind,
    detail: Arc<str>,
    resource_attempt: WorthQueryWorkflowExecutionResourceAttempt,
}

impl WorthQueryManagedWorkflowRunAdmissionFailure {
    pub(super) fn new(
        kind: WorthQueryManagedWorkflowRunAdmissionFailureKind,
        detail: impl Into<Arc<str>>,
        resource_attempt: WorthQueryWorkflowExecutionResourceAttempt,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
            resource_attempt,
        }
    }

    pub fn kind(&self) -> WorthQueryManagedWorkflowRunAdmissionFailureKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn into_resource_attempt(self) -> WorthQueryWorkflowExecutionResourceAttempt {
        self.resource_attempt
    }

    pub fn release(
        self,
    ) -> crate::domain_computation::WorthQueryWorkflowExecutionAttemptReleaseReceipt {
        self.resource_attempt.release()
    }
}

impl std::fmt::Debug for WorthQueryManagedWorkflowRunAdmissionFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryManagedWorkflowRunAdmissionFailure")
            .field("kind", &self.kind)
            .field("detail", &self.detail)
            .field(
                "resource_attempt",
                &self.resource_attempt.resources().identity(),
            )
            .finish()
    }
}
