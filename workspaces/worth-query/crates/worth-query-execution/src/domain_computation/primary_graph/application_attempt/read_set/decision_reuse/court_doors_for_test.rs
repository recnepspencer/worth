//! Sealed test doors over this module's private members.
use super::*;

impl PreparedDecisionReuseContext {
    #[doc(hidden)]
    pub fn new_for_test(
        contract: WorthQueryProducerInputReuseContract,
        key: Option<[u8; 32]>,
        principal: Option<WorthQueryPrincipalReuseWitness>,
        scope: Option<WorthQueryDecisionScopeWitness>,
    ) -> Option<Self> {
        Self::new(contract, key, principal, scope)
    }
}

impl CompletedHandlerFactBoundary {
    #[doc(hidden)]
    pub fn seal_decision_reuse_for_test(
        &self,
        prepared: PreparedDecisionReuseContext,
        actual: DecisionContextUse,
    ) -> Option<CompletedDecisionReuseProof> {
        Self::seal_decision_reuse(self, prepared, actual)
    }
}

impl PreparedDecisionReuseContext {
    #[doc(hidden)]
    pub fn matches_completed_for_test(&self, prior: &CompletedDecisionReuseProof) -> bool {
        Self::matches_completed(self, prior)
    }
}
