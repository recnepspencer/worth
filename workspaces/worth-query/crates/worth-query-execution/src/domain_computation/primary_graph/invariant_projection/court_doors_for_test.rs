//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> WorthQueryApplicationInvariantProjectionSnapshot<Schema> where
    Schema: ApplicationSchema, {
    #[doc(hidden)]
    pub fn into_lease_and_realized_scope_for_test(
        self, product: crate::basis::WorthQueryProductBranchLease) -> (
        super::super::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease,
        WorthQueryRealizedProjectionScope,
        Option<super::super::application_attempt::retained_decision_facts::RetainedSourceFacts>,
        Vec<ConsumedOutputEvidence>,
    ) {
        Self::into_lease_and_realized_scope(self, product)
    }
}
