//! Moved whole from `domain_computation::primary_graph::application_attempt::read_set::decision_reuse`: the items below and every impl they own.
use crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryDecisionScopeWitness;
use crate::domain_computation::vocabulary::primary_graph::input_reuse_contract::WorthQueryDecisionContextDependencies;
use crate::domain_computation::vocabulary::primary_graph::input_reuse_contract::WorthQueryProducerInputReuseContract;
use crate::domain_computation::vocabulary::primary_graph::freshness::WorthQueryPrincipalReuseWitness;
use crate::domain_computation::vocabulary::primary_graph::decision_context_use::DecisionContextUse;
use worth_foundational::facade::DeterminismContract;
use crate::domain_computation::vocabulary::primary_graph::decision_reuse::PreparedDecisionReuseContext;





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
