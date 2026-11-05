//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationQueryGovernance {
    #[doc(hidden)]
    pub fn inspection_matches_for_test(
        &self,
        graph_work: &WorthQueryManagedGraphWorkSession,
        target: &Arc<WorthQueryApplicationInspectionTarget>,
        parameters: &CanonicalDigestId,
        principal: EntityId,
        scope: EntityId,
    ) -> bool {
        Self::inspection_matches(self, graph_work, target, parameters, principal, scope)
    }
}
