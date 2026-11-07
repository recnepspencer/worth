use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::identity_authority::{QueryProjectionIdentity, QuerySubscriptionIdentityKind};

use super::active_error::ActiveSubscriptionLifecycleError;
use super::attachment_error::SubscriptionConsumerAttachmentError;
use super::closeout::SubscriptionLifecycleCloseError;
use super::continuation_error::SubscriptionContinuationError;
use super::delivery_error::QueryDeliveryError;
use super::evidence_projection::subscription_evidence_projection;
use super::preview_isolation_error::PreviewSubscriptionIsolationError;

impl ActiveSubscriptionLifecycleError {
    pub fn source_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.source_identity
    }

    pub fn source_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.source_identity())
    }
}

impl SubscriptionConsumerAttachmentError {
    pub fn source_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.source_identity
    }

    pub fn source_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.source_identity())
    }
}

impl QueryDeliveryError {
    pub fn source_identity(
        &self,
    ) -> Result<
        &WorthQueryEvidenceIdentity,
        crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
    > {
        self.source_identity.as_ref().map_err(|cause| *cause)
    }
    pub fn source_projection(
        &self,
    ) -> Result<
        QueryProjectionIdentity<String, QuerySubscriptionIdentityKind>,
        crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
    > {
        Ok(subscription_evidence_projection(self.source_identity()?))
    }
}

impl SubscriptionContinuationError {
    pub fn source_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.source_identity
    }

    pub fn source_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.source_identity())
    }
}

impl PreviewSubscriptionIsolationError {
    pub fn source_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.source_identity
    }

    pub fn source_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.source_identity())
    }
}

impl SubscriptionLifecycleCloseError {
    pub fn source_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.source_identity
    }

    pub fn source_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.source_identity())
    }
}
