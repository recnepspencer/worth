use crate::source::async_declaration::request_identity::{
    BridgeAsyncRequestIdentityRejection, BridgeAsyncRequestIdentityRejectionKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeAsyncForwardCausalityRejectionKind {
    RetryAdmissionMissing,
    RetryScheduleMissing,
    TimeoutEvidenceMissing,
    CancellationEvidenceMissing,
    RevalidationAdmissionMissing,
    PriorAndNewerDeclarationMismatch,
    PriorAndNewerLoweringMismatch,
    PriorAndNewerFamilyMismatch,
    PriorAndNewerSignalHandleMismatch,
    RequestIdentityAdmissionRejected,
    SignalRuntimeThreadAffinityViolation,
    SignalRuntimeIdentityMismatch,
    StaleSignalGenerationRejected,
    BasisDriftRequiredForRevalidation,
    BasisDriftForbiddenForRetry,
    SubscriptionInstanceDriftForbiddenForRetry,
    SubscriptionInstanceRequiredForSubscriptionBackedFamily,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeAsyncForwardCausalityRejection {
    kind: BridgeAsyncForwardCausalityRejectionKind,
    detail: String,
}

impl BridgeAsyncForwardCausalityRejection {
    pub(crate) fn new(
        kind: BridgeAsyncForwardCausalityRejectionKind,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub fn kind(&self) -> BridgeAsyncForwardCausalityRejectionKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

pub(crate) fn map_request_identity_rejection(
    error: BridgeAsyncRequestIdentityRejection,
) -> BridgeAsyncForwardCausalityRejection {
    let kind = match error.kind() {
        BridgeAsyncRequestIdentityRejectionKind::FamilyKindMismatch => {
            BridgeAsyncForwardCausalityRejectionKind::PriorAndNewerFamilyMismatch
        }
        BridgeAsyncRequestIdentityRejectionKind::LoweringIdentityMismatch => {
            BridgeAsyncForwardCausalityRejectionKind::PriorAndNewerLoweringMismatch
        }
        BridgeAsyncRequestIdentityRejectionKind::SubscriptionInstanceRequired
        | BridgeAsyncRequestIdentityRejectionKind::SubscriptionInstanceUnexpected
        | BridgeAsyncRequestIdentityRejectionKind::PreviewBasisSubscriptionInstanceMismatch => {
            BridgeAsyncForwardCausalityRejectionKind::SubscriptionInstanceRequiredForSubscriptionBackedFamily
        }
        BridgeAsyncRequestIdentityRejectionKind::SignalRuntimeThreadAffinityViolation => {
            BridgeAsyncForwardCausalityRejectionKind::SignalRuntimeThreadAffinityViolation
        }
        BridgeAsyncRequestIdentityRejectionKind::SignalRequestAdmissionRejected
        | BridgeAsyncRequestIdentityRejectionKind::SignalAsyncRequestBlocked
        | BridgeAsyncRequestIdentityRejectionKind::InFlightRequestMissing => {
            BridgeAsyncForwardCausalityRejectionKind::RequestIdentityAdmissionRejected
        }
    };
    rejected(kind, error.detail())
}

pub(crate) fn rejected(
    kind: BridgeAsyncForwardCausalityRejectionKind,
    detail: impl Into<String>,
) -> BridgeAsyncForwardCausalityRejection {
    BridgeAsyncForwardCausalityRejection::new(kind, detail)
}
