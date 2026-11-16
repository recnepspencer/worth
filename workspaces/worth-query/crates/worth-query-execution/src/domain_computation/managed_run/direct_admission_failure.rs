use std::sync::Arc;

use crate::domain_computation::WorthQueryDirectExecutionResourceAttempt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedDirectRunAdmissionFailureKind {
    QueryAuthority,
    DurableCustody,
    ProductBasisMismatch,
    RelationalBasis,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    BridgePlanning,
    InstalledStepContract,
    ReadExecution(
        Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    ),
    BridgeExecutionBasis,
    ManagedAuthorityJoin,
}

pub struct WorthQueryManagedDirectRunAdmissionFailure {
    kind: WorthQueryManagedDirectRunAdmissionFailureKind,
    detail: Arc<str>,
    custody_denial:
        Option<crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial>,
    resource_attempt: WorthQueryDirectExecutionResourceAttempt,
    custody: Option<Arc<super::durable_custody::ManagedRunCustody>>,
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
            custody_denial: None,
            resource_attempt,
            custody: None,
        }
    }

    pub(super) fn with_custody(
        mut self,
        custody: Option<Arc<super::durable_custody::ManagedRunCustody>>,
    ) -> Self {
        self.custody = custody;
        self
    }
    pub(super) fn with_custody_denial(
        mut self,
        denial: crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial,
    ) -> Self {
        self.custody_denial = Some(denial);
        self
    }
    pub fn custody_denial(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial>
    {
        self.custody_denial.as_ref()
    }
    pub fn kind(&self) -> WorthQueryManagedDirectRunAdmissionFailureKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn into_resource_attempt(self) -> WorthQueryDirectExecutionResourceAttempt {
        self.resource_attempt.with_managed_custody(self.custody)
    }

    pub fn release(
        self,
    ) -> crate::domain_computation::WorthQueryDirectExecutionAttemptReleaseReceipt {
        let attempt = self.resource_attempt.attempt_identity().as_str().to_owned();
        let released = self.resource_attempt.release();
        if released.capacity().released_reservation_count() != 0 {
            if let Some(custody) = self.custody {
                custody.record_admission_release(&attempt);
            }
        }
        released
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
#[cfg(test)]
#[path = "direct_admission_failure_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
