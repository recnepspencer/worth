use crate::evidence_identity::WorthQueryEvidenceIdentity;

use super::super::super::evidence_identities::subscription_family_capability_identity;
use super::super::super::family::QuerySubscriptionFamily;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuerySubscriptionSupportClass {
    Declaration,
    Activation,
    ActiveLifecycle,
    Continuation,
    PreviewCloseout,
    DurableReplay,
    StoreBackedRestart,
}

impl QuerySubscriptionSupportClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Declaration => "declaration",
            Self::Activation => "activation",
            Self::ActiveLifecycle => "active_lifecycle",
            Self::Continuation => "continuation",
            Self::PreviewCloseout => "preview_closeout",
            Self::DurableReplay => "durable_replay",
            Self::StoreBackedRestart => "store_backed_restart",
        }
    }

    pub(crate) fn all() -> [Self; 7] {
        [
            Self::Declaration,
            Self::Activation,
            Self::ActiveLifecycle,
            Self::Continuation,
            Self::PreviewCloseout,
            Self::DurableReplay,
            Self::StoreBackedRestart,
        ]
    }

    pub(crate) fn requires_admission_evidence(&self) -> bool {
        matches!(
            self,
            Self::Activation | Self::ActiveLifecycle | Self::Continuation | Self::PreviewCloseout
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuerySubscriptionSupportPosture {
    RuntimeBackedCertified,
    RuntimeBackedDenied,
    RuntimeBackedDeferred,
    UncertifiedDenied,
}

impl QuerySubscriptionSupportPosture {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RuntimeBackedCertified => "runtime_backed_certified",
            Self::RuntimeBackedDenied => "runtime_backed_denied",
            Self::RuntimeBackedDeferred => "runtime_backed_deferred",
            Self::UncertifiedDenied => "uncertified_denied",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionFamilyCapabilityDigest {
    capability_identity: WorthQueryEvidenceIdentity,
}

impl SubscriptionFamilyCapabilityDigest {
    pub(crate) fn for_family(family: &QuerySubscriptionFamily) -> Self {
        Self {
            capability_identity: subscription_family_capability_identity(family),
        }
    }

    pub fn capability_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.capability_identity
    }
}
