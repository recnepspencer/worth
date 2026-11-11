//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryDocumentHistoryOperationConstraint {
    #[doc(hidden)]
    pub fn new_for_test<S, E>(
        scope: &crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<S, E>,
        source: WorthQueryProductObservationLease,
        candidate: WorthQueryProductBranch,
        meaning: WorthQueryApplicationIdempotencyBinding,
        candidate_predecessor: Option<crate::basis::WorthQueryProductBranchReadIdentity>,
    ) -> Self {
        Self {
            operation_scope: HistoryOperationScope::from_resolved(scope),
            source,
            candidate,
            meaning,
            candidate_predecessor,
        }
    }
}
