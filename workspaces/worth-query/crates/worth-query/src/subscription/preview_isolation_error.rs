use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::active_counters::ActiveSubscriptionCounters;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreviewSubscriptionIsolationDenialKind {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    PreviewAuthoritativeSharingDenied,
    PreviewDiscardResidueDenied,
    PreviewLifecycleStateMismatch,
}

impl PreviewSubscriptionIsolationDenialKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EvidenceIdentityDerivationDenied(_) => "evidence_identity_derivation_denied",
            Self::PreviewAuthoritativeSharingDenied => "preview_authoritative_sharing_denied",
            Self::PreviewDiscardResidueDenied => "preview_discard_residue_denied",
            Self::PreviewLifecycleStateMismatch => "preview_lifecycle_state_mismatch",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewSubscriptionIsolationError {
    denial_kind: PreviewSubscriptionIsolationDenialKind,
    message: String,
    pub(in crate::subscription) source_identity: WorthQueryEvidenceIdentity,
    counters: ActiveSubscriptionCounters,
}

impl PreviewSubscriptionIsolationError {
    pub(in crate::subscription) fn identity_refused(
        cause: IdentityDerivationDenial,
        source_identity: WorthQueryEvidenceIdentity,
        counters: ActiveSubscriptionCounters,
    ) -> Self {
        Self::new(
            PreviewSubscriptionIsolationDenialKind::EvidenceIdentityDerivationDenied(cause),
            "canonical digest bounds refused subscription evidence construction",
            source_identity,
            counters,
        )
    }

    pub(super) fn new(
        denial_kind: PreviewSubscriptionIsolationDenialKind,
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

    pub fn denial_kind(&self) -> &PreviewSubscriptionIsolationDenialKind {
        &self.denial_kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn counters(&self) -> &ActiveSubscriptionCounters {
        &self.counters
    }
}
