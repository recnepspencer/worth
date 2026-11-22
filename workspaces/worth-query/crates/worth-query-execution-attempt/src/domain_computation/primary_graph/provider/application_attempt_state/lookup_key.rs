//! Stable session affinity keys for the transient attempt registry.
use super::*;
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct WorthQueryApplicationAttemptLookupKey(WorthQueryProviderSessionAffinityIdentity);

impl WorthQueryApplicationAttemptLookupKey {
    pub(super) fn from_affinity(
        affinity: &crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptAffinity,
    ) -> Self {
        Self(affinity.lookup_identity())
    }

    pub(super) const fn from_identity(identity: WorthQueryProviderSessionAffinityIdentity) -> Self {
        Self(identity)
    }
}
