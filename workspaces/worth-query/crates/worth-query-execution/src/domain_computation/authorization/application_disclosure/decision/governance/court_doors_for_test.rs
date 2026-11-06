//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationQueryGovernance {
    #[doc(hidden)]
    pub fn computation_matches_for_test(
        &self,
        graph_work: &WorthQueryManagedGraphWorkSession,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        query: &WorthQueryInstalledApplicationQueryIdentity,
        parameters: &CanonicalDigestId,
        principal: EntityId,
        scope: EntityId,
    ) -> bool {
        Self::computation_matches(
            self, graph_work, runtime, query, parameters, principal, scope,
        )
    }
}

impl WorthQueryApplicationQueryGovernance {
    #[doc(hidden)]
    pub fn readmission_matches_for_test(
        &self,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        query: &WorthQueryInstalledApplicationQueryIdentity,
        parameters: &CanonicalDigestId,
        principal: EntityId,
        scope: EntityId,
    ) -> bool {
        Self::readmission_matches(self, runtime, query, parameters, principal, scope)
    }
}
