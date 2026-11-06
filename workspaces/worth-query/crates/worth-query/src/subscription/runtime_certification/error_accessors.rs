use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity_authority::{QueryProjectionIdentity, QuerySubscriptionIdentityKind};

use super::super::evidence_projection::subscription_evidence_projection;
use super::error::{
    QuerySubscriptionRuntimeCertificationCounters, QuerySubscriptionRuntimeCertificationError,
};

impl QuerySubscriptionRuntimeCertificationCounters {
    pub fn counter_projection(
        &self,
    ) -> Result<
        QueryProjectionIdentity<String, QuerySubscriptionIdentityKind>,
        IdentityDerivationDenial,
    > {
        Ok(subscription_evidence_projection(&self.counter_identity()?))
    }
}

impl QuerySubscriptionRuntimeCertificationError {
    pub fn failure_identity(
        &self,
    ) -> Result<&crate::evidence_identity::WorthQueryEvidenceIdentity, IdentityDerivationDenial>
    {
        Ok(&self.failure_identity?)
    }

    pub fn failure_projection(
        &self,
    ) -> Result<
        QueryProjectionIdentity<String, QuerySubscriptionIdentityKind>,
        IdentityDerivationDenial,
    > {
        Ok(subscription_evidence_projection(&self.failure_identity?))
    }
}
