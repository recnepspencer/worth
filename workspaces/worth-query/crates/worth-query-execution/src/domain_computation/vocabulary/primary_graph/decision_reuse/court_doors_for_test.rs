//! Sealed test doors over this module's private members.
use crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryDecisionScopeWitness;
use crate::domain_computation::vocabulary::primary_graph::decision_reuse::PreparedDecisionReuseContext;
use crate::domain_computation::vocabulary::primary_graph::freshness::WorthQueryPrincipalReuseWitness;
use crate::domain_computation::vocabulary::primary_graph::input_reuse_contract::WorthQueryProducerInputReuseContract;

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
