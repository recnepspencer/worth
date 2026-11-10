use crate::identity_authority::{QueryProjectionIdentity, QuerySubscriptionIdentityKind};

use super::active_counters::ActiveSubscriptionCounters;
use super::counters::QuerySubscriptionDeclarationCounters;
use super::evidence_projection::subscription_evidence_projection;

impl QuerySubscriptionDeclarationCounters {
    pub fn counter_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.evidence_identity())
    }
}

impl ActiveSubscriptionCounters {
    pub fn counter_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.evidence_identity())
    }
}
