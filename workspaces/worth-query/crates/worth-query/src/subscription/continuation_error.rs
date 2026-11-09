use crate::evidence_identity::WorthQueryEvidenceIdentity;

use super::active_counters::ActiveSubscriptionCounters;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionContinuationDenialKind {
    EvidenceIdentityDerivationDenied(
        crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
    ),
    UnsupportedContinuationClass,
    ContinuationEvidenceMismatch,
    ContinuationRemapBudgetExceeded,
}

impl SubscriptionContinuationDenialKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EvidenceIdentityDerivationDenied(_) => "evidence_identity_derivation_denied",
            Self::UnsupportedContinuationClass => "unsupported_continuation_class",
            Self::ContinuationEvidenceMismatch => "continuation_evidence_mismatch",
            Self::ContinuationRemapBudgetExceeded => "continuation_remap_budget_exceeded",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionContinuationError {
    denial_kind: SubscriptionContinuationDenialKind,
    message: String,
    pub(in crate::subscription) source_identity: WorthQueryEvidenceIdentity,
    counters: ActiveSubscriptionCounters,
}

impl SubscriptionContinuationError {
    pub(super) fn new(
        denial_kind: SubscriptionContinuationDenialKind,
        message: impl Into<String>,
        source_identity: WorthQueryEvidenceIdentity,
        counters: ActiveSubscriptionCounters,
    ) -> Self {
        Self {
            denial_kind,
            message: message.into(),
            source_identity,
            counters,
        }
    }

    pub fn denial_kind(&self) -> &SubscriptionContinuationDenialKind {
        &self.denial_kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn counters(&self) -> &ActiveSubscriptionCounters {
        &self.counters
    }
}
