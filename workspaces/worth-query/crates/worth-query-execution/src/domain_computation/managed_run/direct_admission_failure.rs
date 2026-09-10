use std::sync::Arc;

use crate::domain_computation::WorthQueryDirectExecutionResourceAttempt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedDirectRunAdmissionFailureKind {
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

pub struct WorthQueryManagedDirectRunAdmissionFailure {
    kind: WorthQueryManagedDirectRunAdmissionFailureKind,
    detail: Arc<str>,
    resource_attempt: WorthQueryDirectExecutionResourceAttempt,
}

impl WorthQueryManagedDirectRunAdmissionFailure {
    pub(super) fn new(
        kind: WorthQueryManagedDirectRunAdmissionFailureKind,
        detail: impl Into<Arc<str>>,
        resource_attempt: WorthQueryDirectExecutionResourceAttempt,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
            resource_attempt,
        }
    }

    pub fn kind(&self) -> WorthQueryManagedDirectRunAdmissionFailureKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn into_resource_attempt(self) -> WorthQueryDirectExecutionResourceAttempt {
        self.resource_attempt
    }

    pub fn release(
        self,
    ) -> crate::domain_computation::WorthQueryDirectExecutionAttemptReleaseReceipt {
        self.resource_attempt.release()
    }
}

impl std::fmt::Debug for WorthQueryManagedDirectRunAdmissionFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryManagedDirectRunAdmissionFailure")
            .field("kind", &self.kind)
            .field("detail", &self.detail)
            .field(
                "resource_attempt",
                &self.resource_attempt.resources().identity(),
            )
            .finish()
    }
}
