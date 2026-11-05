use worth_foundational::facade::DeterminismContract;

use crate::domain_computation::vocabulary::primary_graph::handler_fact_boundary::CompletedHandlerFactBoundary;

use crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryDecisionScopeWitness;
use {crate::domain_computation::vocabulary::primary_graph::input_reuse_contract::WorthQueryDecisionContextDependencies, crate::domain_computation::vocabulary::primary_graph::input_reuse_contract::WorthQueryProducerInputReuseContract, crate::domain_computation::vocabulary::primary_graph::freshness::WorthQueryPrincipalReuseWitness, crate::domain_computation::vocabulary::primary_graph::decision_context_use::DecisionContextUse};






#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completed_read_reuse_accepts_tracked_context_and_rejects_raw_or_undeclared_use() {
        let boundary = CompletedHandlerFactBoundary::from_completed_read(2);
        let contract = WorthQueryProducerInputReuseContract::canonical_bitwise(
            WorthQueryDecisionContextDependencies::NONE,
        );
        let capture = || PreparedDecisionReuseContext::new(contract, None, None, None).unwrap();

        let completed = boundary
            .seal_decision_reuse(capture(), DecisionContextUse::default())
            .unwrap();
        // Matching compares recorded context exactly (ee07da2b0e removed its
        // work forecast): an equal empty context matches, a declared key
        // matches only the same key.
        assert!(capture().matches_completed(&completed));
        let declared_key = WorthQueryProducerInputReuseContract::canonical_bitwise(
            WorthQueryDecisionContextDependencies::KEY,
        );
        let key_capture =
            PreparedDecisionReuseContext::new(declared_key, Some([7; 32]), None, None).unwrap();
        let completed_key = boundary
            .seal_decision_reuse(key_capture, DecisionContextUse::default().key_for_test())
            .unwrap();
        assert!(
            PreparedDecisionReuseContext::new(declared_key, Some([7; 32]), None, None)
                .unwrap()
                .matches_completed(&completed_key)
        );
        assert!(!capture().matches_completed(&completed_key));
        assert!(
            !PreparedDecisionReuseContext::new(declared_key, Some([7; 32]), None, None)
                .unwrap()
                .matches_completed(&completed)
        );
        assert!(
            !PreparedDecisionReuseContext::new(declared_key, Some([8; 32]), None, None)
                .unwrap()
                .matches_completed(&completed_key)
        );
        assert!(boundary
            .seal_decision_reuse(
                capture(),
                DecisionContextUse::default().raw_reader_for_test(),
            )
            .is_none());
        assert!(boundary
            .seal_decision_reuse(capture(), DecisionContextUse::default().key_for_test())
            .is_none());
        for untracked in [
            DecisionContextUse::default().managed_computation(),
            DecisionContextUse::default().request_context_for_test(),
        ] {
            assert!(boundary.seal_decision_reuse(capture(), untracked).is_none());
        }
    }
}

mod court_doors_for_test;
