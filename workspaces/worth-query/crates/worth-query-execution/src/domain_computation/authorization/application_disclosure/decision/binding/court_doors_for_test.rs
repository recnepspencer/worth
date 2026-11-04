//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationGovernanceBinding {
    #[doc(hidden)]
    pub fn from_inspection_session_for_test(
        graph_work: &WorthQueryManagedGraphWorkSession,
        target: &Arc<crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationInspectionTarget>,
        parameters: CanonicalDigestId,
        principal: EntityId,
        scope: EntityId,
    ) -> Option<Self> {
        Self::from_inspection_session(graph_work, target, parameters, principal, scope)
    }
}
