//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryDocumentHistoryOperationConstraint {
    #[doc(hidden)]
    pub fn new_for_test(
        model: HistoryModelIdentity,
        source: WorthQueryProductObservationLease,
        candidate: WorthQueryProductBranch,
        meaning: WorthQueryApplicationIdempotencyBinding,
        candidate_predecessor: Option<crate::basis::WorthQueryProductBranchReadIdentity>,
    ) -> Self {
        Self {
            model,
            source,
            candidate,
            meaning,
            candidate_predecessor,
        }
    }
}
