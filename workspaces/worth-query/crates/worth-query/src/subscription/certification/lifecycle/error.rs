use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::super::identity::subscription_certification_failure_identity;
use super::vocabulary::SubscriptionLifecycleCertificationDenialKind;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionLifecycleCertificationError {
    denial_kind: SubscriptionLifecycleCertificationDenialKind,
    message: &'static str,
    failure_identity: WorthQueryEvidenceIdentity,
}

impl SubscriptionLifecycleCertificationError {
    pub(super) fn new(
        denial_kind: SubscriptionLifecycleCertificationDenialKind,
        message: &'static str,
        evidence: &[WorthQueryEvidenceIdentity],
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            denial_kind,
            message,
            failure_identity: subscription_certification_failure_identity(
                "subscription_lifecycle_certification_error_v1",
                denial_kind.as_str(),
                message,
                evidence,
            )?,
        })
    }

    pub fn denial_kind(&self) -> &SubscriptionLifecycleCertificationDenialKind {
        &self.denial_kind
    }

    pub fn message(&self) -> &str {
        self.message
    }

    pub fn failure_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.failure_identity
    }
}
