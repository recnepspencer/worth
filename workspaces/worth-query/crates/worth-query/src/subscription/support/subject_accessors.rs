use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity_authority::{QueryProjectionIdentity, QuerySubscriptionIdentityKind};

use super::super::evidence_projection::subscription_evidence_projection;
use super::matrix::QuerySubscriptionSupportMatrix;
use super::subject::{
    QuerySubscriptionSupportEvidence, QuerySubscriptionSupportEvidenceError,
    QuerySubscriptionSupportSubject, SubscriptionFamilyCapabilityDigest,
};

impl SubscriptionFamilyCapabilityDigest {
    pub fn capability_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.capability_identity())
    }
}

impl QuerySubscriptionSupportMatrix {
    pub fn capability_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        self.capability_digest().capability_projection()
    }
}

impl QuerySubscriptionSupportSubject {
    pub fn declaration_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.declaration_identity())
    }

    pub fn admission_projection(
        &self,
    ) -> Option<QueryProjectionIdentity<String, QuerySubscriptionIdentityKind>> {
        self.admission_identity()
            .map(subscription_evidence_projection)
    }

    pub fn source_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.source_identity())
    }

    pub fn subject_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.subject_identity())
    }
}

impl QuerySubscriptionSupportEvidenceError {
    pub fn failure_projection(
        &self,
    ) -> Result<
        QueryProjectionIdentity<String, QuerySubscriptionIdentityKind>,
        IdentityDerivationDenial,
    > {
        Ok(subscription_evidence_projection(
            self.failure_identity.as_ref().map_err(|cause| *cause)?,
        ))
    }
}

impl QuerySubscriptionSupportEvidence {
    pub(crate) fn source_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.source_identity())
    }
}
