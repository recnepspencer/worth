use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::super::identity::subscription_certification_failure_identity;
use super::vocabulary::SubscriptionLifecycleCertificationDenialKind;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionLifecycleCertificationError {
    denial_kind: SubscriptionLifecycleCertificationDenialKind,
    message: &'static str,
    failure_identity: Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial>,
}

impl SubscriptionLifecycleCertificationError {
    pub(super) fn with_evidence_derivation(
        denial_kind: SubscriptionLifecycleCertificationDenialKind,
        message: &'static str,
        evidence: Result<Vec<WorthQueryEvidenceIdentity>, IdentityDerivationDenial>,
    ) -> Self {
        Self {
            denial_kind,
            message,
            failure_identity: evidence.and_then(|evidence| {
                subscription_certification_failure_identity(
                    "subscription_lifecycle_certification_error_v1",
                    denial_kind.as_str(),
                    message,
                    &evidence,
                )
            }),
        }
    }

    pub(super) fn new(
        denial_kind: SubscriptionLifecycleCertificationDenialKind,
        message: &'static str,
        evidence: &[WorthQueryEvidenceIdentity],
    ) -> Self {
        Self {
            denial_kind,
            message,
            failure_identity: subscription_certification_failure_identity(
                "subscription_lifecycle_certification_error_v1",
                denial_kind.as_str(),
                message,
                evidence,
            ),
        }
    }

    pub fn denial_kind(&self) -> &SubscriptionLifecycleCertificationDenialKind {
        &self.denial_kind
    }

    pub fn message(&self) -> &str {
        self.message
    }

    pub fn failure_identity(
        &self,
    ) -> Result<&WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        self.failure_identity.as_ref().map_err(|cause| *cause)
    }
}

impl From<IdentityDerivationDenial> for SubscriptionLifecycleCertificationError {
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self {
            denial_kind:
                SubscriptionLifecycleCertificationDenialKind::EvidenceIdentityDerivationDenied(
                    cause,
                ),
            message: "subscription lifecycle certification identity refused",
            failure_identity: Err(cause),
        }
    }
}
