use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::active_counters::ActiveSubscriptionCounters;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryDeliveryDenialKind {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    DeliveryWindowBudgetExceeded,
    RawCdcFallbackDenied,
    RawBridgeInvalidationDenied,
    WorkPacketWindowMismatch,
    WorkPacketDeltaMismatch,
    MissingPreviousValueEvidence,
    StaleTemporalBasis,
    DenseRefreshDenied,
    AllocationPostureForbidden,
}

impl QueryDeliveryDenialKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EvidenceIdentityDerivationDenied(_) => "evidence_identity_derivation_denied",
            Self::DeliveryWindowBudgetExceeded => "delivery_window_budget_exceeded",
            Self::RawCdcFallbackDenied => "raw_cdc_fallback_denied",
            Self::RawBridgeInvalidationDenied => "raw_bridge_invalidation_denied",
            Self::WorkPacketWindowMismatch => "work_packet_window_mismatch",
            Self::WorkPacketDeltaMismatch => "work_packet_delta_mismatch",
            Self::MissingPreviousValueEvidence => "missing_previous_value_evidence",
            Self::StaleTemporalBasis => "stale_temporal_basis",
            Self::DenseRefreshDenied => "dense_refresh_denied",
            Self::AllocationPostureForbidden => "allocation_posture_forbidden",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryDeliveryError {
    denial_kind: QueryDeliveryDenialKind,
    message: String,
    pub(in crate::subscription) source_identity:
        Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial>,
    counters: ActiveSubscriptionCounters,
}

impl QueryDeliveryError {
    pub(in crate::subscription) fn identity_refused(
        cause: IdentityDerivationDenial,
        source_identity: WorthQueryEvidenceIdentity,
        counters: ActiveSubscriptionCounters,
    ) -> Self {
        Self::new(
            QueryDeliveryDenialKind::EvidenceIdentityDerivationDenied(cause),
            "canonical digest bounds refused subscription evidence construction",
            source_identity,
            counters,
        )
    }

    pub(super) fn with_source_derivation(
        denial_kind: QueryDeliveryDenialKind,
        message: impl Into<String>,
        source_identity: Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial>,
        counters: ActiveSubscriptionCounters,
    ) -> Self {
        Self {
            denial_kind,
            message: message.into(),
            source_identity,
            counters,
        }
    }

    pub(super) fn new(
        denial_kind: QueryDeliveryDenialKind,
        message: impl Into<String>,
        source_identity: WorthQueryEvidenceIdentity,
        counters: ActiveSubscriptionCounters,
    ) -> Self {
        Self {
            denial_kind,
            message: message.into(),
            source_identity: Ok(source_identity),
            counters,
        }
    }

    pub fn denial_kind(&self) -> &QueryDeliveryDenialKind {
        &self.denial_kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn counters(&self) -> &ActiveSubscriptionCounters {
        &self.counters
    }
}
