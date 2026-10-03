use worth_foundational::facade::DeterminismContract;

use super::CompletedHandlerFactBoundary;

use crate::domain_computation::authorization::WorthQueryDecisionScopeWitness;
use crate::domain_computation::primary_graph::{
    application_contribution::{
        WorthQueryDecisionContextDependencies, WorthQueryProducerInputReuseContract,
    },
    freshness::WorthQueryPrincipalReuseWitness,
    DecisionContextUse,
};

/// Invocation evidence awaiting the completed, exactly covered handler read.
/// The static producer declaration supplies the upper bound on context use.
pub(in crate::domain_computation::primary_graph) struct PreparedDecisionReuseContext {
    contract: WorthQueryProducerInputReuseContract,
    key: Option<[u8; 32]>,
    principal: Option<WorthQueryPrincipalReuseWitness>,
    scope: Option<WorthQueryDecisionScopeWitness>,
}

/// Recorded fixed native and canonical context for a completed producer read.
/// An absent proof is ineligible; the handler fact count remains independent.
pub(in crate::domain_computation::primary_graph) struct CompletedDecisionReuseProof {
    key: Option<[u8; 32]>,
    principal: Option<WorthQueryPrincipalReuseWitness>,
    scope: Option<WorthQueryDecisionScopeWitness>,
}

impl PreparedDecisionReuseContext {
    pub(in crate::domain_computation::primary_graph) fn new(
        contract: WorthQueryProducerInputReuseContract,
        key: Option<[u8; 32]>,
        principal: Option<WorthQueryPrincipalReuseWitness>,
        scope: Option<WorthQueryDecisionScopeWitness>,
    ) -> Option<Self> {
        if contract.determinism() != DeterminismContract::CanonicalBitwise {
            return None;
        }
        let declared = contract.context();
        if declared.contains(WorthQueryDecisionContextDependencies::KEY) != key.is_some()
            || declared.contains(WorthQueryDecisionContextDependencies::PRINCIPAL)
                != principal.is_some()
            || declared.contains(WorthQueryDecisionContextDependencies::SCOPE) != scope.is_some()
        {
            return None;
        }
        Some(Self {
            contract,
            key,
            principal,
            scope,
        })
    }

    pub(super) fn seal_after_completed_read(
        self,
        actual: DecisionContextUse,
    ) -> Option<CompletedDecisionReuseProof> {
        if actual.has_untracked_context()
            || actual.declared_context_bits() & !self.contract.context().bits() != 0
        {
            return None;
        }
        Some(CompletedDecisionReuseProof {
            key: self.key,
            principal: self.principal,
            scope: self.scope,
        })
    }

    /// A fresh invocation can compare its descriptive admitted context with
    /// prior completed evidence without pretending the new handler ran.
    pub(in crate::domain_computation::primary_graph) fn matches_completed<E>(
        &self,
        prior: &CompletedDecisionReuseProof,
        mut admit: impl FnMut(u64) -> Result<(), E>,
    ) -> Result<bool, E> {
        let work = 1
            + u64::from(self.key.is_some())
            + if self.principal.is_some() { 10 } else { 0 }
            + if self.scope.is_some() { 5 } else { 0 };
        admit(work)?;
        Ok(self.key == prior.key && self.principal == prior.principal && self.scope == prior.scope)
    }
}

impl CompletedHandlerFactBoundary {
    /// Only the exactly completed read can turn a descriptive capture into
    /// reusable decision evidence. The handler fact prefix stays separate.
    pub(in crate::domain_computation::primary_graph) fn seal_decision_reuse(
        &self,
        prepared: PreparedDecisionReuseContext,
        actual: DecisionContextUse,
    ) -> Option<CompletedDecisionReuseProof> {
        prepared.seal_after_completed_read(actual)
    }
}

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
        let mut charged = 0;
        assert!(capture()
            .matches_completed(&completed, |work| {
                charged += work;
                Ok::<_, ()>(())
            })
            .unwrap());
        assert_eq!(charged, 1);
        let declared_key = WorthQueryProducerInputReuseContract::canonical_bitwise(
            WorthQueryDecisionContextDependencies::KEY,
        );
        let key_capture =
            PreparedDecisionReuseContext::new(declared_key, Some([7; 32]), None, None).unwrap();
        let completed_key = boundary
            .seal_decision_reuse(key_capture, DecisionContextUse::default().key_for_test())
            .unwrap();
        assert!(
            !PreparedDecisionReuseContext::new(declared_key, Some([8; 32]), None, None)
                .unwrap()
                .matches_completed(&completed_key, |_| Ok::<_, ()>(()))
                .unwrap()
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
    }
}
