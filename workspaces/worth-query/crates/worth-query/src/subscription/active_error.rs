use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::active_counters::ActiveSubscriptionCounters;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActiveSubscriptionLifecycleDenialKind {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    WorkBudgetExceeded,
    HeapAllocationForbidden,
    DurableCheckpointOverclaim,
    StoreBackedRestartOverclaim,
    RegistryEquivalenceMismatch,
    AttachmentNotActive,
    AttachmentLaneMismatch,
    LinearScanLookupForbidden,
}

impl ActiveSubscriptionLifecycleDenialKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EvidenceIdentityDerivationDenied(_) => "evidence_identity_derivation_denied",
            Self::WorkBudgetExceeded => "work_budget_exceeded",
            Self::HeapAllocationForbidden => "heap_allocation_forbidden",
            Self::DurableCheckpointOverclaim => "durable_checkpoint_overclaim",
            Self::StoreBackedRestartOverclaim => "store_backed_restart_overclaim",
            Self::RegistryEquivalenceMismatch => "registry_equivalence_mismatch",
            Self::AttachmentNotActive => "attachment_not_active",
            Self::AttachmentLaneMismatch => "attachment_lane_mismatch",
            Self::LinearScanLookupForbidden => "linear_scan_lookup_forbidden",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveSubscriptionLifecycleError {
    denial_kind: ActiveSubscriptionLifecycleDenialKind,
    message: String,
    pub(in crate::subscription) source_identity: WorthQueryEvidenceIdentity,
    counters: ActiveSubscriptionCounters,
}

impl ActiveSubscriptionLifecycleError {
    pub(in crate::subscription) fn identity_refused(
        cause: IdentityDerivationDenial,
        source_identity: WorthQueryEvidenceIdentity,
        counters: ActiveSubscriptionCounters,
    ) -> Self {
        Self::new(
            ActiveSubscriptionLifecycleDenialKind::EvidenceIdentityDerivationDenied(cause),
            "canonical digest bounds refused subscription evidence construction",
            source_identity,
            counters,
        )
    }

    pub(super) fn new(
        denial_kind: ActiveSubscriptionLifecycleDenialKind,
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

    pub fn denial_kind(&self) -> &ActiveSubscriptionLifecycleDenialKind {
        &self.denial_kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn counters(&self) -> &ActiveSubscriptionCounters {
        &self.counters
    }
}
